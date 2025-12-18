# Paperless-ngx Uploader

A secure Rust CLI tool for uploading documents to your Paperless-ngx instance. Features batch uploads, file filtering, automatic archiving, and secure credential storage.

## Features

- 📤 **Batch uploads** - Upload multiple files or entire folders
- 🔍 **Regex filtering** - Filter files using regular expressions
- 🗄️ **Automatic archiving** - Move uploaded files to archive folder
- 🧹 **Cleanup** - Automatically delete old archived files
- 🔐 **Secure credentials** - Tokens stored in OS keyring, never in plaintext

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