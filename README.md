# fanotify-perm-fix

**fanotify-perm-fix** is a robust, protocol-agnostic daemon written in Rust. It leverages the Linux fanotify API to monitor a specified base directory (and all of its subdirectories) for file system events such as file creation, moves, or attribute changes. Based on a TOML configuration file, the daemon automatically fixes file/directory ownership and permission modes—ensuring that files added locally, via Samba, NFS, or any other protocol, always conform to the desired security settings.

## Features
- **Protocol Agnostic:**
  Monitors low-level fanotify events, so it works regardless of whether files are created locally, via Samba, or via NFS.

- **Configurable Behavior:**
  Loads a TOML configuration file at startup that lets you independently control how owner, group, and permission modes are handled. For example, you can specify that the daemon should inherit the owner and group from the parent directory while always forcing fixed permission modes for files and directories.

- **Robust and Safe:**
  Designed with careful error handling and resource management. It only modifies file metadata (using chown/chmod) without touching file contents, reducing the risk of data loss or corruption.

- **Deployable as a Systemd Service:**
  Easily integrated into your system startup.

## Requirements
- Linux with fanotify support (kernel 4.5 or later)
- Rust toolchain installed
- Root privileges (fanotify requires CAP_SYS_ADMIN)

## Installation
1. Clone the Repository:
```
bash
git clone https://github.com/yourusername/fanotify-perm-fix.git
cd fanotify-perm-fix
```
2. Build the Project:
```
bash
cargo build --release
```
This produces a binary at target/release/fanotify_perm_fix.

3. Install the Binary:
Copy the binary to a system path (e.g., /usr/local/bin/):
```
bash
sudo cp target/release/fanotify_perm_fix /usr/local/bin/fanotify_perm_fix
```

## Configuration
Create a TOML configuration file at /etc/fanotify-perm-fix.toml. The configuration lets you specify, for each attribute, whether to inherit the value from the parent directory or to force a fixed value. For example:

```
toml
[settings]
# Base directory to monitor recursively.
base_dir = "/data/samba/share"

# For each attribute, if true the daemon will inherit from the parent.
# Otherwise, it will use the fixed value provided.
inherit_owner = true
inherit_group = true
inherit_permissions = false

# Fixed values to use if inheritance is disabled.
# Leave fixed_owner empty ("") to not force a user change.
fixed_owner = ""
fixed_group = "sambashare"

# Fixed permission modes for files and directories (octal as a string).
fixed_file_permission = "0664"
fixed_directory_permission = "2775"

# When inheriting permissions, parent's mode will be ANDed with this mask.
permission_mask = "0666"

# Log level: e.g., "info", "debug"
log_level = "info"
```
In the above configuration, owner and group are inherited from the parent, while permission modes are always set to fixed values.

## Usage
Run the daemon as root:
```
bash
sudo /usr/local/bin/fanotify_perm_fix
```
It will monitor the directory specified in base_dir for events and automatically adjust ownership and permissions as configured.

## Running as a Systemd Service
To ensure that the daemon starts at boot, create a systemd unit file at /etc/systemd/system/fanotify-perm-fix.service:
```
ini
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
Then reload systemd and enable the service:
```
bash

sudo systemctl daemon-reload
sudo systemctl enable fanotify-perm-fix
sudo systemctl start fanotify-perm-fix
```

## Contributing
Contributions, bug reports, and feature requests are welcome. Please feel free to open issues or submit pull requests.

## License
This project is licensed under the MIT License. See the LICENSE file for details.
