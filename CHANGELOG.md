## 0.2.0 - Security Hardening & Documentation

### ✨ New Features

- Added HTTPS enforcement option for Paperless-ngx endpoint connections with HTTP fallback for development and debugging purposes, including warning messages

### 🛠️ Improvements

- Enhanced CLI token handling to prevent exposure in process listings and shell history
- Added comprehensive docstrings to `get_endpoint_by_prompt()` and `get_token_by_prompt()` functions in input.rs
- Documented `Client::upload()`, `Client::upload_files()`, and `Client::upload_file()` methods with complete usage information
- Documented `setup_keyring()` function explaining service name, entry name, and platform-specific behavior
- Added advanced usage examples for automation scenarios: cron jobs, systemd timers, non-interactive environment variable usage, and batch processing scripts
- Added project badges and Paperless-ngx version compatibility information to README.md

### 🐛 Bug Fixes

- Fixed authentication token exposure in logs by removing sensitive credential logging from `get_token_by_prompt()` and main.rs debug output
- Corrected misleading log message in token input function that incorrectly labeled token input as endpoint input

---

## What's Changed

- security: Fixed authentication token exposure in logs by @contributor in src/cmd/input.rs
- security: Removed token logging from main.rs debug output by @contributor in src/main.rs
- feature: Added HTTPS enforcement option with HTTP fallback for development by @contributor in src/cmd/config/mod.rs
- security: Enhanced CLI token handling to prevent shell history exposure by @contributor in src/cmd/input.rs
- docs: Added docstrings to input.rs user prompt functions by @contributor in src/cmd/input.rs
- docs: Added comprehensive documentation to Client upload methods by @contributor in src/client.rs
- docs: Documented keyring setup infrastructure by @contributor in src/cmd/config/keyring.rs
- docs: Added advanced automation usage examples to README by @contributor in README.md
- docs: Added project badges and compatibility information by @contributor in README.md
