pub mod permission {
    use std::fs;
    use std::io;
    use std::path::Path;
    use nix::sys::stat::{chmod, Mode};
    use nix::unistd::{chown, Uid, Gid};
    use log::info;
    use crate::config::Settings;
    use crate::utils::{parse_octal, get_uid_from_username, get_gid_from_group};

    /// Computes the new attributes (owner, group, mode) for the given file or directory
    /// based on the provided settings.
    /// 
    /// For owner and group, if the respective inherit flag is true, the parent's attribute is used;
    /// otherwise, if a fixed value is provided, that fixed value is used.
    /// For permissions, if inherit_permissions is false, the fixed permission mode (differing for files
    /// and directories) is used. (If inherit_permissions is true, parent's mode is inherited and masked.)
    pub fn compute_new_attributes(path: &str, settings: &Settings) -> io::Result<(Option<Uid>, Option<Gid>, Mode)> {
        let path_obj = Path::new(path);
        let parent = path_obj.parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No parent directory"))?;
        let parent_meta = fs::metadata(parent)?;

        // Determine new owner.
        let new_uid = if settings.inherit_owner {
            Some(Uid::from_raw(parent_meta.uid()))
        } else if let Some(ref fixed_owner) = settings.fixed_owner {
            if fixed_owner.is_empty() { None } else { get_uid_from_username(fixed_owner) }
        } else {
            None
        };

        // Determine new group.
        let new_gid = if settings.inherit_group {
            Some(Gid::from_raw(parent_meta.gid()))
        } else if let Some(ref fixed_group) = settings.fixed_group {
            if fixed_group.is_empty() { None } else { get_gid_from_group(fixed_group) }
        } else {
            None
        };

        let meta = fs::metadata(path)?;
        let is_dir = meta.is_dir();

        let mode = if settings.inherit_permissions {
            // Inherit parent's mode and apply mask.
            let parent_mode = parent_meta.mode();
            let mask = parse_octal(&settings.permission_mask)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid permission_mask"))?;
            Mode::from_bits(parent_mode & mask)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid inherited mode"))?
        } else {
            // Use fixed mode.
            let fixed_perm_str = if is_dir {
                &settings.fixed_directory_permission
            } else {
                &settings.fixed_file_permission
            };
            let fixed_perm = parse_octal(fixed_perm_str)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid fixed permission value"))?;
            let mut mode = Mode::from_bits(fixed_perm)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid mode bits"))?;
            // For directories, ensure setgid is set.
            if is_dir && (mode.bits() & 0o2000 == 0) {
                mode |= Mode::from_bits_truncate(0o2000);
            }
            mode
        };

        Ok((new_uid, new_gid, mode))
    }

    /// Applies the new attributes to the given file or directory.
    pub fn fix_permissions(path: &str, settings: &Settings) -> io::Result<()> {
        let (new_uid, new_gid, new_mode) = compute_new_attributes(path, settings)?;
        chown(path, new_uid, new_gid)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("chown failed: {}", e)))?;
        chmod(path, new_mode)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("chmod failed: {}", e)))?;
        info!("Fixed {}: mode set to {:o}", path, new_mode.bits());
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs::{File, create_dir_all};
        use tempfile::tempdir;
        use crate::config::Settings;

        // Create dummy settings for testing "fixed" mode.
        fn dummy_settings_fixed() -> Settings {
            Settings {
                base_dir: "/tmp".to_string(),
                inherit_owner: false,
                inherit_group: false,
                inherit_permissions: false,
                fixed_owner: None,
                fixed_group: Some("sambashare".to_string()),
                fixed_file_permission: "0664".to_string(),
                fixed_directory_permission: "2775".to_string(),
                permission_mask: "0666".to_string(),
                log_level: Some("info".to_string()),
            }
        }

        #[test]
        fn test_compute_new_attributes_fixed_file() {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("testfile");
            File::create(&file_path).unwrap();
            
            let settings = dummy_settings_fixed();
            let (uid, gid, mode) = compute_new_attributes(file_path.to_str().unwrap(), &settings).unwrap();
            assert_eq!(mode.bits(), 0o0664);
            // In fixed mode with inheritance flags false, owner remains None and group is forced.
            assert!(uid.is_none());
            assert!(gid.is_some());
        }
    }
}
