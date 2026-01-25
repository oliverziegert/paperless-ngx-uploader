# Paperless-ngx Uploader

![Build Status](https://github.com/oliverziegert/paperless-ngx-uploader/actions/workflows/test.yaml/badge.svg)
![Version](https://img.shields.io/github/v/release/oliverziegert/paperless-ngx-uploader)
![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)

A secure Rust CLI tool for uploading documents to your Paperless-ngx instance. Features batch uploads, file filtering, automatic archiving, and secure credential storage.

## Features

- 📤 **Batch uploads** - Upload multiple files or entire folders
- 🔍 **Regex filtering** - Filter files using regular expressions
- 🗄️ **Automatic archiving** - Move uploaded files to archive folder
- 🧹 **Cleanup** - Automatically delete old archived files
- 🔐 **Secure credentials** - Tokens stored in OS keyring, never in plaintext

## Compatibility

This tool uses the standard Paperless-ngx REST API and is compatible with:

- **Paperless-ngx**: v1.10.0 and later
- **API Version**: Stable document upload endpoint (`/api/documents/post_document/`)

The uploader has been tested with recent Paperless-ngx versions and should work with any version that supports the standard document upload API. If you encounter compatibility issues with your Paperless-ngx version, please [open an issue](https://github.com/oliverziegert/paperless-ngx-uploader/issues).

## Installation

```bash
# Build from source
git clone https://github.com/oliverziegert/paperless-ngx-uploader.git
cd paperless-ngx-uploader
cargo build --release

# Binary will be available at ./target/release/paperless-ngx-uploader
```

## Configuration

### Split Configuration Architecture

The application uses a secure split configuration approach:

- **Public configuration** (endpoint URL): Stored in `~/.config/paperless-ngx-uploader/config.yaml`
- **Private configuration** (token): Stored in your OS keyring
  - macOS: Keychain
  - Linux: Secret Service
  - Windows: Credential Manager

This ensures your authentication token is never stored in plaintext.

### Initial Setup

Initialize your configuration with the `init` command:

```bash
paperless-ngx-uploader init
```

You'll be prompted for:
1. **Paperless-ngx URL** - The endpoint of your Paperless-ngx instance (e.g., `http://localhost:8000`)
2. **Authentication Token** - Your API token from Paperless-ngx settings

### Viewing Configuration

Check where your configuration is stored:

```bash
# View config file location
ls ~/.config/paperless-ngx-uploader/config.yaml

# View current endpoint (token is securely stored in keyring)
cat ~/.config/paperless-ngx-uploader/config.yaml
```

### Updating Configuration

To update your endpoint or token, simply run `init` again:

```bash
paperless-ngx-uploader init
```

### Removing Configuration

To completely remove all configuration (both file and keyring):

```bash
# Remove config file
rm ~/.config/paperless-ngx-uploader/config.yaml

# Remove token from keyring (platform-specific)
# macOS: Open Keychain Access and search for "paperless-ngx-uploader"
# Linux: Use your distribution's keyring manager
# Windows: Open Credential Manager and search for "paperless-ngx-uploader"
```

## Usage

### Upload Single File

```bash
paperless-ngx-uploader upload --file /path/to/document.pdf
```

### Upload Multiple Files

```bash
paperless-ngx-uploader upload --file doc1.pdf --file doc2.pdf --file doc3.pdf
```

### Upload Folder

```bash
paperless-ngx-uploader upload --folder /path/to/documents
```

### Upload with Regex Filter

Upload only files matching a pattern:

```bash
paperless-ngx-uploader upload --folder /path/to/documents --regex "invoice.*\.pdf"
```

### Upload with Archiving

Move uploaded files to an `archive/` subfolder:

```bash
paperless-ngx-uploader upload --folder /path/to/documents --archive
```

### Upload with Cleanup

Delete archived files older than 31 days (default):

```bash
paperless-ngx-uploader upload --folder /path/to/documents --archive --cleanup
```

### Custom Cleanup Period

Delete archived files older than 7 days:

```bash
paperless-ngx-uploader upload --folder /path/to/documents --archive --cleanup --cleanup-after-days 7
```

### Custom Config Path

Use a different config file location:

```bash
paperless-ngx-uploader --config /custom/path/config.yaml upload --file document.pdf
```

## Advanced Usage - Automation

For automated workflows, you'll need to ensure the uploader can run non-interactively without prompting for credentials.

### Non-Interactive Setup

Before setting up automation, initialize the configuration once interactively:

```bash
# Run this once to store credentials securely
paperless-ngx-uploader init
```

This stores your token in the OS keyring, allowing the uploader to access it automatically in future runs.

### Cron Jobs

Schedule automatic uploads using cron. This example uploads files daily at 2 AM:

```bash
# Edit your crontab
crontab -e

# Add this line to upload daily at 2:00 AM
0 2 * * * /path/to/paperless-ngx-uploader upload --folder /home/user/scans --archive --cleanup

# Upload every 4 hours with regex filtering
0 */4 * * * /path/to/paperless-ngx-uploader upload --folder /home/user/documents --regex ".*\.pdf$" --archive

# Upload on weekdays at 9 AM with custom cleanup period
0 9 * * 1-5 /path/to/paperless-ngx-uploader upload --folder /home/user/invoices --archive --cleanup --cleanup-after-days 7
```

**Tip:** Use absolute paths in cron jobs and redirect output to a log file for debugging:

```bash
0 2 * * * /usr/local/bin/paperless-ngx-uploader upload --folder /home/user/scans --archive >> /var/log/paperless-upload.log 2>&1
```

### Systemd Service and Timer

For more control and better logging, use systemd. Create a service that runs on a timer:

**Service file** (`~/.config/systemd/user/paperless-upload.service`):

```ini
[Unit]
Description=Upload documents to Paperless-ngx
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
ExecStart=/path/to/paperless-ngx-uploader upload --folder /home/user/scans --archive --cleanup
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
```

**Timer file** (`~/.config/systemd/user/paperless-upload.timer`):

```ini
[Unit]
Description=Run Paperless-ngx upload daily

[Timer]
# Run daily at 2:00 AM
OnCalendar=daily
OnCalendar=*-*-* 02:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

**Enable and start the timer:**

```bash
# Reload systemd to recognize new files
systemctl --user daemon-reload

# Enable timer to start on boot
systemctl --user enable paperless-upload.timer

# Start the timer now
systemctl --user start paperless-upload.timer

# Check timer status
systemctl --user status paperless-upload.timer

# View logs
journalctl --user -u paperless-upload.service -f
```

### Batch Processing Script

For complex workflows, create a shell script that orchestrates multiple upload operations:

**Example:** `upload-batch.sh`

```bash
#!/bin/bash

# Exit on any error
set -e

# Configuration
UPLOADER="/path/to/paperless-ngx-uploader"
LOG_FILE="/var/log/paperless-batch-upload.log"

# Function to log with timestamp
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log "Starting batch upload process"

# Upload invoices with 7-day cleanup
log "Processing invoices..."
$UPLOADER upload --folder /home/user/invoices --regex "invoice.*\.pdf" --archive --cleanup --cleanup-after-days 7

# Upload receipts with 30-day cleanup
log "Processing receipts..."
$UPLOADER upload --folder /home/user/receipts --archive --cleanup --cleanup-after-days 30

# Upload general documents with 60-day cleanup
log "Processing general documents..."
$UPLOADER upload --folder /home/user/documents --archive --cleanup --cleanup-after-days 60

log "Batch upload completed successfully"
```

**Make executable and run:**

```bash
chmod +x upload-batch.sh
./upload-batch.sh
```

### Watch Folder (Continuous Monitoring)

Monitor a folder for new files and upload them immediately using `inotifywait` (Linux):

```bash
#!/bin/bash

WATCH_DIR="/home/user/scan-inbox"
UPLOADER="/path/to/paperless-ngx-uploader"

# Install inotify-tools if needed: sudo apt-get install inotify-tools

echo "Watching $WATCH_DIR for new files..."

inotifywait -m -e close_write -e moved_to "$WATCH_DIR" --format '%w%f' | while read FILE
do
    echo "New file detected: $FILE"
    $UPLOADER upload --file "$FILE" --archive
done
```

**Run as systemd service** (`~/.config/systemd/user/paperless-watch.service`):

```ini
[Unit]
Description=Watch folder for Paperless-ngx uploads
After=network-online.target

[Service]
Type=simple
ExecStart=/path/to/watch-folder.sh
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
```

### Docker Integration

If running Paperless-ngx in Docker, you can mount a volume and use the uploader from the host:

```bash
# Upload to Paperless-ngx running in Docker
paperless-ngx-uploader upload --folder /host/path/to/documents --archive

# Or use Docker Compose to add a sidecar service (docker-compose.yml)
```

```yaml
services:
  paperless-upload:
    image: rust:latest
    volumes:
      - ./paperless-ngx-uploader:/app
      - /host/scans:/scans
    command: /app/target/release/paperless-ngx-uploader upload --folder /scans --archive --cleanup
    environment:
      - PAPERLESS_URL=http://paperless:8000
```

## Troubleshooting

### "Token not configured" error

Run the init command to set up your authentication:

```bash
paperless-ngx-uploader init
```

### "Keyring not available" error

Your system may not have a supported keyring service. Ensure you have:
- **macOS**: Keychain (built-in)
- **Linux**: `gnome-keyring` or compatible Secret Service provider
- **Windows**: Credential Manager (built-in)

### Connection errors

Verify your Paperless-ngx instance is running and accessible:

```bash
curl http://your-paperless-instance:8000/api/
```

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run ignored keyring tests (requires real keyring)
cargo test -- --ignored
```

### Enable Debug Logging

```bash
RUST_LOG=debug paperless-ngx-uploader upload --file test.pdf
```

## Security

- Authentication tokens are **never stored in plaintext**
- Tokens are stored using your operating system's native credential manager
- Configuration files only contain non-sensitive data (endpoint URLs)

## License

See LICENSE file for details.

## Contributing

Contributions are welcome! Please open an issue or pull request.