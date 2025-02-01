pub mod utils {
    use std::ffi::CString;
    use std::io;
    use std::fs;
    use nix::unistd::{Uid, Gid};

    /// Parse an octal string (e.g. "0664") into a u32.
    pub fn parse_octal(s: &str) -> Option<u32> {
        u32::from_str_radix(s, 8).ok()
    }

    /// Lookup the UID corresponding to the given username.
    pub fn get_uid_from_username(username: &str) -> Option<Uid> {
        let cstr = CString::new(username).ok()?;
        unsafe {
            let pwd = libc::getpwnam(cstr.as_ptr());
            if pwd.is_null() { None } else { Some(Uid::from_raw((*pwd).pw_uid)) }
        }
    }

    /// Lookup the GID corresponding to the given group name.
    pub fn get_gid_from_group(group: &str) -> Option<Gid> {
        let cstr = CString::new(group).ok()?;
        unsafe {
            let grp = libc::getgrnam(cstr.as_ptr());
            if grp.is_null() { None } else { Some(Gid::from_raw((*grp).gr_gid)) }
        }
    }

    /// Convert a file descriptor to its corresponding path using /proc/self/fd/<fd>.
    pub fn fd_to_path(fd: i32) -> io::Result<String> {
        let fd_path = format!("/proc/self/fd/{}", fd);
        fs::read_link(fd_path).map(|pb| pb.to_string_lossy().into_owned())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_parse_octal() {
            assert_eq!(parse_octal("0664"), Some(0o664));
            assert_eq!(parse_octal("2775"), Some(0o2775));
        }
    }
}
