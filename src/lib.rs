use std::io;

use std::fs::File;

use std::os::fd::AsRawFd;
use std::os::fd::RawFd;

use std::mem::MaybeUninit;

pub struct UnlockedFd(RawFd);

impl From<&File> for UnlockedFd {
    fn from(f: &File) -> Self {
        let raw: RawFd = f.as_raw_fd();
        Self(raw)
    }
}

#[derive(Clone, Copy)]
pub enum FlockMode {
    Shared,
    SharedNonBlocking,
    Exclusive,
    ExclusiveNonBlocking,
}

impl FlockMode {
    pub fn into_libc_flags(self) -> libc::c_int {
        match self {
            Self::Shared => libc::LOCK_SH,
            Self::SharedNonBlocking => libc::LOCK_SH | libc::LOCK_NB,
            Self::Exclusive => libc::LOCK_EX,
            Self::ExclusiveNonBlocking => libc::LOCK_EX | libc::LOCK_NB,
        }
    }
}

impl FlockMode {
    pub fn to_prot(&self) -> libc::c_int {
        match self {
            Self::Shared | Self::SharedNonBlocking => libc::PROT_READ,
            Self::Exclusive | Self::ExclusiveNonBlocking => libc::PROT_READ | libc::PROT_WRITE,
        }
    }
}

impl From<FlockMode> for libc::c_int {
    fn from(m: FlockMode) -> Self {
        m.into_libc_flags()
    }
}

pub struct LockedFd(RawFd);

impl Drop for LockedFd {
    fn drop(&mut self) {
        let fd: RawFd = self.0;
        #[allow(unsafe_code)]
        let res: libc::c_int = unsafe { libc::flock(fd, libc::LOCK_UN) };
        if 0 != res {
            eprintln!("UNABLE TO UNLOCK: {}", io::Error::last_os_error())
        }
    }
}

impl UnlockedFd {
    pub fn into_locked(self, mode: FlockMode) -> Result<LockedFd, io::Error> {
        let fd: RawFd = self.0;
        let fmode: libc::c_int = mode.into();
        #[allow(unsafe_code)]
        let ret: libc::c_int = unsafe { libc::flock(fd, fmode) };
        match ret {
            0 => Ok(LockedFd(fd)),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

pub struct ReadonlyMmap {
    ptr: *mut std::ffi::c_void,
    len: usize,
    lfd: LockedFd,
}

impl ReadonlyMmap {
    pub fn ino(&self) -> Result<u64, io::Error> {
        self.lfd.ino()
    }
}

impl ReadonlyMmap {
    pub fn as_slice(&self) -> &[u8] {
        let ptr: *const u8 = self.ptr as *const u8;
        let len: usize = self.len;
        #[allow(unsafe_code)]
        unsafe {
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

impl ReadonlyMmap {
    pub fn count_lines<F>(&self, mut counter: F) -> Result<usize, io::Error>
    where
        F: FnMut(&[u8]) -> Result<usize, io::Error>,
    {
        let dat: &[u8] = self.as_slice();
        let mut chunks = dat.chunks(65536);
        chunks.try_fold(0, |state, next| {
            let subtot: usize = counter(next)?;
            Ok(state + subtot)
        })
    }
}

impl ReadonlyMmap {
    pub fn count_lines_native(&self) -> Result<usize, io::Error> {
        self.count_lines(|chnk: &[u8]| {
            let filtered = chnk.iter().filter(|&b| 0x0a.eq(b));
            Ok(filtered.count())
        })
    }
}

impl Drop for ReadonlyMmap {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        let ret: libc::c_int = unsafe { libc::munmap(self.ptr, self.len) };

        if 0 != ret {
            eprintln!("UNABLE TO MUNMAP: {}", io::Error::last_os_error())
        }
    }
}

impl LockedFd {
    pub fn len(&self) -> Result<usize, io::Error> {
        let raw: RawFd = self.0;
        let mut mstat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
        #[allow(unsafe_code)]
        let ret: libc::c_int = unsafe { libc::fstat(raw, mstat.as_mut_ptr()) };
        if 0 != ret {
            return Err(io::Error::last_os_error());
        }

        #[allow(unsafe_code)]
        let stat: libc::stat = unsafe { mstat.assume_init() };

        Ok(stat.st_size as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.len().map(|sz| 0 == sz).unwrap_or_default()
    }

    pub fn ino(&self) -> Result<u64, io::Error> {
        let raw: RawFd = self.0;
        let mut mstat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
        #[allow(unsafe_code)]
        let ret: libc::c_int = unsafe { libc::fstat(raw, mstat.as_mut_ptr()) };
        if 0 != ret {
            return Err(io::Error::last_os_error());
        }

        #[allow(unsafe_code)]
        let stat: libc::stat = unsafe { mstat.assume_init() };

        Ok(stat.st_ino)
    }

    pub fn into_map(self) -> Result<ReadonlyMmap, io::Error> {
        let size: usize = self.len()?;
        if 0 == size {
            return Err(io::Error::other("empty file"));
        }

        let raw: RawFd = self.0;
        let prot: libc::c_int = libc::PROT_READ;
        #[allow(unsafe_code)]
        let ptr: *mut std::ffi::c_void =
            unsafe { libc::mmap(std::ptr::null_mut(), size, prot, libc::MAP_SHARED, raw, 0) };

        if libc::MAP_FAILED == ptr {
            return Err(io::Error::last_os_error());
        }

        Ok(ReadonlyMmap {
            ptr,
            len: size,
            lfd: self,
        })
    }
}
