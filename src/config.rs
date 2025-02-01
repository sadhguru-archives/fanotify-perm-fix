pub mod config {
    use serde::Deserialize;
    use std::fs;
    use std::io;

    #[derive(Deserialize)]
    pub struct Settings {
        pub base_dir: String,
        // For owner: if true, inherit parent's owner; otherwise use fixed_owner (if provided).
        pub inherit_owner: bool,
        // For group: if true, inherit parent's group; otherwise use fixed_group.
        pub inherit_group: bool,
        // For permissions: if true, inherit parent's mode and apply permission_mask;
        // if false, use the fixed permission modes.
        pub inherit_permissions: bool,

        // Fixed values (if not inheriting). Empty string means "do not force".
        pub fixed_owner: Option<String>,
        pub fixed_group: Option<String>,

        // Fixed permission modes for files and directories (octal strings).
        pub fixed_file_permission: String,
        pub fixed_directory_permission: String,

        // When inheriting permissions, mask to adjust parent's mode (octal string).
        pub permission_mask: String,

        // Log level (e.g., "info", "debug").
        pub log_level: Option<String>,
    }

    #[derive(Deserialize)]
    pub struct Config {
        pub settings: Settings,
    }

    /// Loads the TOML configuration from the specified file path.
    pub fn load_config(path: &str) -> io::Result<Config> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_load_config() {
            let sample = r#"
            [settings]
            base_dir = "/data/samba/share"
            inherit_owner = true
            inherit_group = true
            inherit_permissions = false
            fixed_owner = ""
            fixed_group = "sambashare"
            fixed_file_permission = "0664"
            fixed_directory_permission = "2775"
            permission_mask = "0666"
            log_level = "info"
            "#;
            let cfg: Config = toml::from_str(sample).unwrap();
            assert_eq!(cfg.settings.base_dir, "/data/samba/share");
            assert!(cfg.settings.inherit_owner);
            assert!(cfg.settings.inherit_group);
            assert!(!cfg.settings.inherit_permissions);
        }
    }
}
