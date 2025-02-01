// src/main.rs

//! Fanotify Permission Fix Daemon
//! 
//! This daemon monitors a specified base directory (recursively) for file creation,
//! moves, and attribute-change events using fanotify. It loads its configuration
//! from a TOML file (e.g. `/etc/fanotify-perm-fix.toml`) at startup and caches it.
//! For each event, the daemon computes new attributes (owner, group, mode)
//! based on per-attribute flags:
//! 
//! - If `inherit_owner` (or `inherit_group`) is true, the affected file inherits that
//!   attribute from its parent directory; otherwise, a fixed value (if provided) is used.
//! - For permissions, if `inherit_permissions` is false then fixed permission modes (for
//!   files and directories separately) are forced.
//! - (If `inherit_permissions` were true, a mask could be applied to the parent’s mode.)
//! 
//! The daemon logs actions and errors, and it is designed to adjust metadata only,
//! without modifying file contents. It is protocol agnostic and works for local,
//! SMB, NFS, etc.
//!
//! This program must run as root.

use std::io;
use std::process::exit;
use std::thread;
use std::time::Duration;

use log::{info, error};

mod config;
mod fanotify;
mod permission;
mod utils;

use config::{load_config, Config};
use fanotify::FanotifyWatcher;
use permission::fix_permissions;

fn main() -> io::Result<()> {
    // Initialize logging.
    env_logger::init();

    // Load configuration once at startup.
    let config_path = "/etc/fanotify-perm-fix.toml";
    let config = match load_config(config_path) {
        Ok(cfg) => cfg.settings,
        Err(e) => {
            eprintln!("Failed to load configuration from {}: {}", config_path, e);
            exit(1);
        }
    };

    info!("Starting fanotify-perm-fix daemon on base directory: {}", config.base_dir);

    // Initialize the fanotify watcher.
    let mut watcher = FanotifyWatcher::new(&config.base_dir)?;
    watcher.mark()?;

    // Buffer for reading events.
    let mut buf = vec![0u8; 4096];

    loop {
        let nread = watcher.read_events(&mut buf)?;
        if nread == 0 {
            thread::sleep(Duration::from_millis(100));
            continue;
        }
        // Process each event in the buffer.
        watcher.process_events(&buf[..nread], |metadata| {
            if metadata.fd >= 0 {
                match utils::fd_to_path(metadata.fd) {
                    Ok(path) => {
                        info!("Event on {} (mask: {:#x})", path, metadata.mask);
                        if let Err(e) = fix_permissions(&path, &config) {
                            error!("Failed to fix permissions on {}: {}", path, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to resolve fd {}: {}", metadata.fd, e);
                    }
                }
                // Close the event file descriptor.
                unsafe { libc::close(metadata.fd); }
            }
            Ok(())
        })?;
    }
}

