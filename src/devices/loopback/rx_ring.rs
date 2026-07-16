use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{devices::RxBufDesc, protocols::ip::IP_PAYLOAD_SIZE_MAX};

#[derive(Debug)]
struct LoPacketBuf {
    buf: Option<[u8; IP_PAYLOAD_SIZE_MAX]>,
    written_size: usize,
}

impl LoPacketBuf {
    const fn new() -> Self {
        Self {
            buf: None,
            written_size: 0,
        }
    }
}

const LOOPBACK_DEVICE_NR_BUFS: usize = 64;

// NIC の DMA をユーザランドで再現
#[derive(Debug)]
pub(super) struct LoRxRing {
    // @next_to_write
    // @end_of_writable
    // @next_to_clean
    // index of buffer
    //
    // 0 <= next_to_write, end_of_writable, next_to_clean < ETHER_TAP_DEVICE_PAGE_MAX * buf_per_page()
    //
    // In a cyclical sense, next_to_write < end_of_writable <= next_to_clean must be always satisfied.
    next_to_write: AtomicUsize,
    end_of_writable: AtomicUsize,
    next_to_clean: AtomicUsize,

    packet_bufs: [UnsafeCell<LoPacketBuf>; LOOPBACK_DEVICE_NR_BUFS],
}

impl LoRxRing {
    pub(super) fn new() -> Self {
        Self {
            next_to_write: AtomicUsize::new(1),
            end_of_writable: AtomicUsize::new(0),
            next_to_clean: AtomicUsize::new(1),
            packet_bufs: [const { UnsafeCell::new(LoPacketBuf::new()) }; LOOPBACK_DEVICE_NR_BUFS],
        }
    }

    /// buf_to_clean() returns buf desc and buf when some packets received.
    /// This does not proceed any index.
    /// You must call `free_buf(desc)` after packet cleaning finished by `desc`.
    ///
    /// You must logically guarantee only single thread calls `buf_to_clean()` for SAFETY
    pub(crate) unsafe fn buf_to_clean(&self) -> Option<(RxBufDesc, &[u8])> {
        let next_to_write = self.next_to_write.load(Ordering::Acquire);
        let next_to_clean = self.next_to_clean.load(Ordering::Acquire);

        if next_to_clean == next_to_write {
            // buffer underflow
            None
        } else {
            unsafe {
                let pk_buf = &*self.packet_bufs[next_to_clean].get();

                Some((
                    RxBufDesc::new(next_to_clean),
                    &pk_buf.buf.as_ref().unwrap().as_slice()[..pk_buf.written_size],
                ))
            }
        }
    }

    /// free buffers by `desc`.
    ///
    /// NOTE: buffers will be freed **by** `desc` from `end_of_writable`.
    /// You must call this after cleaning all of buffers of received packets certainly finished.
    ///
    /// You must logically guarantee only single thread calls `free_buf()` for SAFETY
    pub(crate) unsafe fn free_buf(&self, desc: RxBufDesc) {
        self.next_to_clean
            .update(Ordering::Release, Ordering::Acquire, |next_to_clean| {
                if next_to_clean + 1 == LOOPBACK_DEVICE_NR_BUFS {
                    0
                } else {
                    next_to_clean + 1
                }
            });
        let next_to_write = self.next_to_write.load(Ordering::Acquire);
        self.end_of_writable
            .update(Ordering::Release, Ordering::Acquire, |end_of_writable| {
                // check cyclical correct
                if next_to_write < end_of_writable
                    && (end_of_writable <= desc.value() || desc.value() <= next_to_write)
                    || end_of_writable <= desc.value() && desc.value() <= next_to_write
                {
                    desc.value()
                } else {
                    end_of_writable
                }
            });
    }

    /// You must logically guarantee only single thread calls `buf_to_write()` for SAFETY
    pub(crate) unsafe fn buf_to_write<'a>(&'a self) -> Option<LoRxRingWriteBuf<'a>> {
        let end_of_writable = self.end_of_writable.load(Ordering::Acquire);
        let next_to_write = self.next_to_write.load(Ordering::Acquire);

        if next_to_write == end_of_writable {
            // buffer overflow
            None
        } else {
            unsafe {
                let pk_buf = &mut *self.packet_bufs[next_to_write].get();

                Some(LoRxRingWriteBuf {
                    buf: pk_buf.buf.get_or_insert([0; IP_PAYLOAD_SIZE_MAX]).as_mut(),
                    next_to_write: &self.next_to_write,
                    size: &mut pk_buf.written_size,
                })
            }
        }
    }
}

// SAFETY: ユーザが buf_to_write(), buf_to_clean(), free_buf() という unsafe fn を
// それぞれ1箇所でしか呼ばれていないことを責任を持って保証する限り、スレッド安全である
unsafe impl Send for LoRxRing {}
unsafe impl Sync for LoRxRing {}

pub(super) struct LoRxRingWriteBuf<'a> {
    pub(super) buf: &'a mut [u8],
    next_to_write: &'a AtomicUsize,
    size: &'a mut usize,
}

impl<'a> LoRxRingWriteBuf<'a> {
    pub(super) fn commit(self, size: usize) {
        *self.size = size;

        self.next_to_write
            .update(Ordering::Release, Ordering::Acquire, |next_to_write| {
                if next_to_write + 1 == LOOPBACK_DEVICE_NR_BUFS {
                    0
                } else {
                    next_to_write + 1
                }
            });
    }
}
