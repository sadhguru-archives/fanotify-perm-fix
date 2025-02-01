pub mod fanotify {
    use std::ffi::CString;
    use std::io;
    use std::os::unix::io::RawFd;
    use std::ptr;
    use std::slice;
    use std::thread;
    use std::time::Duration;
    use log::{info, error};
    use libc;

    // Import fanotify constants from the parent module.
    use super::{FAN_CLASS_NOTIF, FAN_NONBLOCK, FAN_CLOEXEC, FAN_MARK_ADD, FAN_EVENT_ON_CHILD, FAN_CREATE, FAN_MOVED_TO, FAN_ATTRIB};

    #[repr(C)]
    pub struct FanotifyEventMetadata {
        pub event_len: u32,
        pub vers: u8,
        pub reserved: u8,
        pub metadata_len: u16,
        pub mask: u64,
        pub fd: i32,
        pub pid: i32,
    }

    pub struct FanotifyWatcher {
        fd: RawFd,
        pub base_dir: String,
    }

    impl FanotifyWatcher {
        /// Create a new fanotify watcher for the given base directory.
        pub fn new(base_dir: &str) -> io::Result<FanotifyWatcher> {
            let fd = unsafe {
                libc::fanotify_init(
                    FAN_CLASS_NOTIF | FAN_NONBLOCK | FAN_CLOEXEC,
                    libc::O_RDONLY | libc::O_LARGEFILE,
                )
            };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(FanotifyWatcher {
                fd,
                base_dir: base_dir.to_string(),
            })
        }

        /// Mark the base directory (and its children) for events.
        pub fn mark(&self) -> io::Result<()> {
            let mark_flags = FAN_MARK_ADD | FAN_EVENT_ON_CHILD;
            let base_dir_c = CString::new(self.base_dir.clone()).unwrap();
            let res = unsafe {
                libc::fanotify_mark(
                    self.fd,
                    mark_flags,
                    FAN_CREATE | FAN_MOVED_TO | FAN_ATTRIB,
                    -1,
                    base_dir_c.as_ptr(),
                )
            };
            if res < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }

        /// Read events from the fanotify file descriptor into the provided buffer.
        pub fn read_events(&self, buf: &mut [u8]) -> io::Result<usize> {
            let n = unsafe {
                libc::read(
                    self.fd,
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                )
            };
            if n < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(n as usize)
        }

        /// Process events in the buffer using the provided closure.
        /// The closure is called with each FanotifyEventMetadata.
        pub fn process_events<F>(&self, buf: &[u8], mut process_fn: F) -> io::Result<()>
        where
            F: FnMut(&FanotifyEventMetadata) -> io::Result<()>,
        {
            let mut offset = 0;
            while offset < buf.len() {
                if offset + std::mem::size_of::<FanotifyEventMetadata>() > buf.len() {
                    break;
                }
                let metadata: FanotifyEventMetadata = unsafe {
                    std::ptr::read(buf.as_ptr().add(offset) as *const FanotifyEventMetadata)
                };
                if metadata.event_len == 0 {
                    break;
                }
                process_fn(&metadata)?;
                offset += metadata.event_len as usize;
            }
            Ok(())
        }
    }

    impl Drop for FanotifyWatcher {
        fn drop(&mut self) {
            unsafe { libc::close(self.fd); }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use tempfile::tempdir;
        use std::fs::create_dir_all;

        #[test]
        fn test_fanotify_watcher_mark() {
            let dir = tempdir().unwrap();
            let base_dir = dir.path().to_str().unwrap();
            // Create a nested directory.
            create_dir_all(dir.path().join("subdir")).unwrap();
            let watcher = FanotifyWatcher::new(base_dir).unwrap();
            assert!(watcher.mark().is_ok());
        }
    }
}
