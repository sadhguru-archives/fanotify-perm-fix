# fanotify-perm-fix

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A robust, protocol-agnostic daemon written in Rust that automatically fixes file/directory permissions using Linux fanotify API.

## 🚀 Features

- **Protocol Agnostic**: Monitors low-level fanotify events, working seamlessly with local operations, Samba, NFS, and other protocols

- **Configurable Behavior**: Uses TOML configuration for flexible control over ownership, groups, and permissions

- **Robust and Safe**: Implements careful error handling and resource management, only modifying file metadata

- **Systemd Integration**: Easy deployment as a system service

## 📋 Requirements

- Linux kernel 4.5 or later (with fanotify support)
- Rust toolchain
- Root privileges (CAP_SYS_ADMIN capability)

## 🔧 Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/fanotify-perm-fix.git
   cd fanotify-perm-fix
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the binary:
   ```bash
   sudo cp target/release/fanotify_perm_fix /usr/local/bin/fanotify_perm_fix
   ```

## ⚙️ Configuration

Create a TOML configuration file at `/etc/fanotify-perm-fix.toml`:

```toml
[settings]
# Base directory to monitor recursively
base_dir = "/data/samba/share"

# Inheritance settings
inherit_owner = true
inherit_group = true
inherit_permissions = false

# Fixed values (when inheritance is disabled)
fixed_owner = ""
fixed_group = "sambashare"

# Permission modes (octal as string)
fixed_file_permission = "0664"
fixed_directory_permission = "2775"

# Permission inheritance mask
permission_mask = "0666"

# Logging configuration
log_level = "info"
```

## 🚦 Usage

Run the daemon with root privileges:

```bash
sudo /usr/local/bin/fanotify_perm_fix
```

### Running as a Systemd Service

1. Create a systemd unit file at `/etc/systemd/system/fanotify-perm-fix.service`:

   ```ini
   [Unit]
   Description=Fanotify Permission Fix Daemon
   After=network.target

   [Service]
   Type=simple
   ExecStart=/usr/local/bin/fanotify_perm_fix
   Restart=on-failure
   User=root

   [Install]
   WantedBy=multi-user.target
   ```

2. Enable and start the service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable fanotify-perm-fix
   sudo systemctl start fanotify-perm-fix
   ```

## 👥 Contributing

We welcome contributions! Feel free to:
- Open issues for bug reports or feature requests
- Submit pull requests
- Improve documentation

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
*Made with ❤️ by the fanotify-perm-fix team*
