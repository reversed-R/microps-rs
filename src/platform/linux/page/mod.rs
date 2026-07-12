use std::{ffi::c_void, ptr::NonNull, sync::LazyLock};

static PAGE_SIZE: LazyLock<usize> = LazyLock::new(|| {
    let ps = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    assert!(ps > 0);

    ps as usize
});

// ページの再現
#[derive(Debug)]
pub(super) struct Page {
    ptr: NonNull<u8>,
}

impl Page {
    pub(super) fn alloc() -> Self {
        let ptr: *mut c_void = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                *PAGE_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            panic!("failed to mmap()");
        }

        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut u8) },
        }
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), *PAGE_SIZE) }
    }

    pub(super) fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), *PAGE_SIZE) }
    }

    #[inline]
    pub(super) fn size() -> usize {
        *PAGE_SIZE
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.ptr.as_ptr().cast(), *PAGE_SIZE) };
    }
}

unsafe impl Send for Page {}
unsafe impl Sync for Page {}
