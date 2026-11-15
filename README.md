# Paperless-ngx Uploader

![Build Status](https://github.com/oliverziegert/paperless-ngx-uploader/actions/workflows/ci.yaml/badge.svg?branch=main)
![Version](https://img.shields.io/github/v/release/oliverziegert/paperless-ngx-uploader)
![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)

A secure Rust CLI tool for uploading documents to your Paperless-ngx instance, built to get scans from a document scanner into Paperless-ngx reliably. Features batch uploads, file filtering, automatic archiving, and secure credential storage.

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
- **`status` command**: needs Paperless-ngx 2.1.0 or newer (it probes `/api/profile/`); `upload` is unaffected

The uploader has been tested with recent Paperless-ngx versions and should work with any version that supports the standard document upload API. If you encounter compatibility issues with your Paperless-ngx version, please [open an issue](https://github.com/oliverziegert/paperless-ngx-uploader/issues).

## Why This Tool Exists

My document scanner (Fujitsu ScanSnap iX1500) can only save scans to local storage or to
cloud services. It cannot upload to something useful like
WebDAV, let alone to Paperless-ngx directly.

This tool closes that gap. It was built to get scanned letters into
Paperless-ngx reliably:

1. The scanner triggers a scan job on a VM.
2. The VM stores the scanned document as a PDF in a temporary folder.
3. The VM runs `paperless-ngx-uploader` to push the PDF to Paperless-ngx.

The ScanSnap driver and software are only available for macOS and Windows,
so the VM runs Windows and calls the uploader like this:

```powershell
C:\path\to\binary\paperless-ngx-uploader.exe upload --folder C:\path\to\scans --archive --delete
```

With `--archive --delete`, uploaded scans are kept in an `archive/` subfolder
as a safety net and deleted after 31 days (the default `--period`).

It works just as well for any other folder of documents; see
[Usage](#usage) and [Advanced Usage - Automation](#advanced-usage---automation).

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

- **Public configuration** (base URL of your Paperless-ngx instance): Stored in `~/.config/paperless-ngx-uploader/config.yaml`
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
1. **Paperless-ngx URL** - The base URL of your Paperless-ngx instance (e.g., `https://paperless.example.com`, or `https://example.com/paperless` for a sub-path install). Do not add an API path; the uploader appends `/api/...` itself. If you paste a full API URL, `init` stores the base URL and tells you.
2. **Authentication Token** - Your API token from Paperless-ngx settings

### Viewing Configuration

Check where your configuration is stored:

```bash
# View config file location
ls ~/.config/paperless-ngx-uploader/config.yaml

# View current endpoint, i.e. the base URL (token is securely stored in keyring)
cat ~/.config/paperless-ngx-uploader/config.yaml
```

### Updating Configuration

To update your endpoint (base URL) or token, simply run `init` again:

```bash
paperless-ngx-uploader init
```

### Upgrading to 0.4.0

- The endpoint in `config.yaml` is now the **base URL** of your Paperless-ngx instance (e.g. `https://paperless.example.com`, or `https://example.com/paperless` for a sub-path install). `upload` appends `/api/documents/post_document/` and `status` appends `/api/profile/`. Before 0.4.0, `upload` posted to the endpoint exactly as stored, so a base URL failed with `403 Forbidden` ([#269](https://github.com/oliverziegert/paperless-ngx-uploader/issues/269)).
- Existing configs are corrected automatically: an endpoint ending in `/api/documents/post_document`, `/api/documents` or `/api` is rewritten to the base URL on the first run, and a notice on stderr names the old and the new value. No action is needed.
- The automatic rewrite will be **removed in 0.6.0**. From then on such an endpoint is an error.
- If the config file is read-only (e.g. a read-only Docker mount), it cannot be rewritten. The command still runs with the corrected value but repeats the notice on every run; change the endpoint to the base URL by hand.
- `init` accepts either form and always stores the base URL.

Before (0.3.x workaround):

```yaml
endpoint: https://paperless.example.com/api/documents/post_document/
allow_insecure: false
```

After (0.4.0 and later):

```yaml
endpoint: https://paperless.example.com
allow_insecure: false
```

Your token stays in the OS keyring; there is no need to run `init` again.

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

`--file` takes exactly one file. To upload several, put them in a folder and select them with `--filter`:

```bash
paperless-ngx-uploader upload --folder /path/to/documents --filter "doc[123]\.pdf"
```

### Upload Folder

```bash
paperless-ngx-uploader upload --folder /path/to/documents
```

### Upload with Regex Filter

Upload only files matching a pattern:

```bash
paperless-ngx-uploader upload --folder /path/to/documents --filter "invoice.*\.pdf"
```

### Upload with Archiving

Move uploaded files to an `archive/` subfolder:

```bash
paperless-ngx-uploader upload --folder /path/to/documents --archive
```

### Upload with Cleanup

Delete archived files older than 31 days (default):

```bash
paperless-ngx-uploader upload --folder /path/to/documents --archive --delete
```

### Custom Cleanup Period

Delete archived files older than 7 days:

```bash
paperless-ngx-uploader upload --folder /path/to/documents --archive --delete --period 7
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
0 2 * * * /path/to/paperless-ngx-uploader upload --folder /home/user/scans --archive --delete

# Upload every 4 hours with regex filtering
0 */4 * * * /path/to/paperless-ngx-uploader upload --folder /home/user/documents --filter ".*\.pdf$" --archive

# Upload on weekdays at 9 AM with custom cleanup period
0 9 * * 1-5 /path/to/paperless-ngx-uploader upload --folder /home/user/invoices --archive --delete --period 7
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
ExecStart=/path/to/paperless-ngx-uploader upload --folder /home/user/scans --archive --delete
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
$UPLOADER upload --folder /home/user/invoices --filter "invoice.*\.pdf" --archive --delete --period 7

# Upload receipts with 30-day cleanup
log "Processing receipts..."
$UPLOADER upload --folder /home/user/receipts --archive --delete --period 30

# Upload general documents with 60-day cleanup
log "Processing general documents..."
$UPLOADER upload --folder /home/user/documents --archive --delete --period 60

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
    command: /app/target/release/paperless-ngx-uploader upload --folder /scans --archive --delete
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
- Configuration files only contain non-sensitive data (the base URL of your instance)

## License

See LICENSE file for details.

## Contributing

Contributions are welcome! Please open an issue or pull request.