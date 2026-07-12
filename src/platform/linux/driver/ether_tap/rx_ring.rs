use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    devices::{RxBufDesc, ethernet},
    platform::linux::{driver::ether_tap::EtherTapDevice, page::Page},
};

#[derive(Debug)]
struct PageBuf {
    page: Option<Page>,
}

impl PageBuf {
    const fn new() -> Self {
        Self { page: None }
    }
}

#[derive(Debug, Clone)]
struct RxRingBufInfo {
    size: usize,
}

impl RxRingBufInfo {
    const fn new(size: usize) -> Self {
        Self { size }
    }
}

// NIC の DMA をユーザランドで再現
#[derive(Debug)]
pub(super) struct EtherTapRxRing {
    // @next_to_write
    // @end_of_writable
    // @next_to_clean
    // index of buffer
    // Page is used as multiple buffer when buf_per_page() > 1
    // so, next_to_write / buf_per_page() is real index of dma_pages
    // and (next_to_write % buf_per_page()) * BUFSIZE is offset in page
    //
    // 0 <= next_to_write, end_of_writable, next_to_clean < ETHER_TAP_DEVICE_PAGE_MAX * buf_per_page()
    //
    // In a cyclical sense, next_to_write < end_of_writable <= next_to_clean must be always satisfied.
    next_to_write: AtomicUsize,
    end_of_writable: AtomicUsize,
    next_to_clean: AtomicUsize,

    dma_pages: [UnsafeCell<PageBuf>; EtherTapDevice::ETHER_TAP_DEVICE_PAGE_MAX],
    buf_descs: [UnsafeCell<RxRingBufInfo>; EtherTapDevice::ETHER_TAP_DEVICE_PAGE_MAX],
}

impl EtherTapRxRing {
    const BUFSIZE: usize = ethernet::ETHER_FRAME_SIZE_MAX.next_power_of_two();

    pub(super) fn new() -> Self {
        Self {
            next_to_write: AtomicUsize::new(1),
            end_of_writable: AtomicUsize::new(0),
            next_to_clean: AtomicUsize::new(1),
            dma_pages: [const { UnsafeCell::new(PageBuf::new()) };
                EtherTapDevice::ETHER_TAP_DEVICE_PAGE_MAX],
            buf_descs: [const { UnsafeCell::new(RxRingBufInfo::new(0)) };
                EtherTapDevice::ETHER_TAP_DEVICE_PAGE_MAX],
        }
    }

    fn buf_per_page() -> usize {
        Page::size() / Self::BUFSIZE
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
            let clean_idx = next_to_clean / Self::buf_per_page();
            let clean_offset = (next_to_clean % Self::buf_per_page()) * Self::BUFSIZE;

            let size = unsafe { (*self.buf_descs[next_to_clean].get()).size };

            unsafe {
                Some((
                    RxBufDesc::new(next_to_clean),
                    &(&mut *self.dma_pages[clean_idx].get())
                        .page
                        .as_ref()
                        .unwrap()
                        .as_slice()[clean_offset..clean_offset + size],
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
    pub(crate) unsafe fn buf_to_write<'a>(&'a self) -> Option<EtherTapRxRingWriteBuf<'a>> {
        let end_of_writable = self.end_of_writable.load(Ordering::Acquire);
        let next_to_write = self.next_to_write.load(Ordering::Acquire);

        if next_to_write == end_of_writable {
            // buffer overflow
            None
        } else {
            let write_idx = next_to_write / Self::buf_per_page();
            let write_offset = (next_to_write % Self::buf_per_page()) * Self::BUFSIZE;

            unsafe {
                Some(EtherTapRxRingWriteBuf {
                    buf: &mut (&mut *self.dma_pages[write_idx].get())
                        .page
                        .as_mut()
                        .unwrap()
                        .as_slice_mut()[write_offset..],
                    next_to_write: &self.next_to_write,
                    buf_info: &mut *self.buf_descs[next_to_write].get(),
                })
            }
        }
    }
}

// SAFETY: ユーザが buf_to_write(), buf_to_clean(), free_buf() という unsafe fn を
// それぞれ1箇所でしか呼ばれていないことを責任を持って保証する限り、スレッド安全である
unsafe impl Send for EtherTapRxRing {}
unsafe impl Sync for EtherTapRxRing {}

pub(super) struct EtherTapRxRingWriteBuf<'a> {
    pub(super) buf: &'a mut [u8],
    next_to_write: &'a AtomicUsize,
    buf_info: &'a mut RxRingBufInfo,
}

impl<'a> EtherTapRxRingWriteBuf<'a> {
    pub(super) fn commit(self, size: usize) {
        self.buf_info.size = size;

        self.next_to_write
            .update(Ordering::Release, Ordering::Acquire, |next_to_write| {
                if next_to_write + 1
                    == EtherTapDevice::ETHER_TAP_DEVICE_PAGE_MAX * EtherTapRxRing::buf_per_page()
                {
                    0
                } else {
                    next_to_write + 1
                }
            });
    }
}
