use core::{cmp, ptr, fmt, mem};

pub const MAX_SIZE: usize = 64;

#[derive(Clone)]
pub struct Buffer {
    inner: core::mem::MaybeUninit<[u8; MAX_SIZE]>,
    len: u8,
}

impl Buffer {
    #[inline]
    ///Creates new instance
    pub const fn new() -> Self {
        Self {
            inner: core::mem::MaybeUninit::uninit(),
            len: 0,
        }
    }

    #[inline]
    ///Returns number of bytes that can be written.
    pub const fn available_len(&self) -> usize {
        MAX_SIZE - self.len as usize
    }

    #[inline]
    ///Returns pointer  to the beginning of underlying buffer
    pub const fn as_ptr(&self) -> *const u8 {
        &self.inner as *const _ as *const u8
    }

    ///Writes bytes.
    ///
    ///If there is not enough space, returns `false`
    ///Otherwise `true`.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> bool {
        if bytes.len() > self.available_len() {
            return false;
        } else if bytes.len() != 0 {
            let ptr = unsafe {
                self.as_ptr().offset(self.len as isize) as *mut u8
            };
            self.len += bytes.len() as u8;

            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            }
        }

        true
    }

    #[inline(always)]
    fn as_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.as_ptr(), self.len as usize)
        }
    }

    #[inline(always)]
    ///Access str from underlying storage
    ///
    ///Returns empty if nothing has been written into buffer yet.
    pub fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(self.as_slice())
        }
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.as_str())
    }
}

impl cmp::PartialEq for Buffer {
    fn eq(&self, other: &Buffer) -> bool {
        self.len == other.len && self.as_slice() == other.as_slice()
    }
}

impl cmp::Eq for Buffer {
}

impl fmt::Write for Buffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.write_bytes(s.as_bytes()) {
            true => Ok(()),
            false => Err(fmt::Error::default())
        }
    }
}
