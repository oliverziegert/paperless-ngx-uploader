## 0.4.0

### ⚠️ Breaking

- The endpoint in `config.yaml` is now the **base URL** of your Paperless-ngx instance (e.g. `https://paperless.example.com`, or `https://example.com/paperless` for a sub-path install). `upload` appends `/api/documents/post_document/` and `status` appends `/api/`. Before 0.4.0, `upload` posted to the endpoint exactly as stored, so a base URL failed with `403 Forbidden` ([#269](https://github.com/oliverziegert/paperless-ngx-uploader/issues/269)).
- Existing configs are corrected automatically: an endpoint ending in `/api/documents/post_document`, `/api/documents` or `/api` is rewritten to the base URL on the first run, and a notice on stderr names the old and the new value. No action is needed.
- The automatic rewrite will be **removed in 0.6.0**. From then on such an endpoint is an error.
- If the config file is read-only (e.g. a read-only Docker mount), it cannot be rewritten. The command still runs with the corrected value but repeats the notice on every run; change the endpoint to the base URL by hand.
- `init` accepts either form and always stores the base URL.

Releases 0.2.3 to 0.3.24 are not listed here; see the [GitHub releases](https://github.com/oliverziegert/paperless-ngx-uploader/releases).

---

## 0.2.2 - Enhanced User Experience & Developer Quality

### ✨ New Features

- Add status command for connectivity verification with configured Paperless-ngx instance
- Add dry-run mode to upload command to preview operations without making changes
- Add recursive folder scanning option to support nested directory structures
- Add upload summary report with statistics showing success/failure counts and file details

### 🛠️ Improvements

- Implement visual progress bar for batch uploads using indicatif crate
- Add colorized terminal output to differentiate success, error, warning, and info messages
- Enhance interactive prompts with input validation, examples, and contextual help messages
- Adjust default logging behavior to show meaningful user feedback without verbose flags
- Add endpoint URL validation during init command to catch common configuration errors
- Reduce string allocations in `get_title_from_filename()` function
- Cache compiled regex for file filtering to eliminate redundant compilation
- Optimize archive folder existence checks in `delete_expired_files()` to avoid redundant operations

### 🔧 Other Changes

- Fix test isolation by using tempfile for all file operations instead of working directory
- Add CI/CD pipeline with GitHub Actions linting and formatting checks
- Remove unused `ConfigCommand` enum and `APP_VERSION` constant
- Evaluate lighter-weight alternatives to inquire crate to reduce binary size

---

## What's Changed

- feat: add status command for connectivity verification
- feat: add dry-run mode to upload command
- feat: add recursive folder scanning option
- feat: add upload summary report with statistics
- feat: implement visual progress bar for batch uploads
- feat: add colorized terminal output for status messages
- feat: enhance interactive prompts with validation and help
- improvement: adjust default verbose level for user feedback
- improvement: add endpoint URL validation in init command
- improvement: reduce string allocations in filename processing
- improvement: cache compiled regex for file filtering
- improvement: optimize archive folder existence checks
- fix: use tempfile for test isolation in client tests
- ci: add GitHub Actions CI/CD pipeline
- refactor: remove unused code and dead code paths
- refactor: evaluate lighter alternatives to inquire crate

## 0.2.1 - Improved Error Handling and Async Performance

### ✨ New Features

- Parallel file uploads using async/await with tokio runtime for significantly improved batch upload throughput

### 🛠️ Improvements

- Configured HTTP client with explicit timeout settings to prevent indefinite hangs and slowloris-style attacks
- Refactored `src/cmd/client.rs` to split concerns into separate modules, reducing file size from 510 lines to under 500-line limit per project guidelines

### 🐛 Bug Fixes

- Replaced `panic!()` calls in production code paths with graceful error handling in config directory creation (src/cmd/config/mod.rs) and HTTP client initialization (src/cmd/client.rs)
- Replaced `.unwrap()` calls on `Option<OsStr>` in file path handling with proper error propagation
- Fixed potential information disclosure by removing system paths and detailed error information from panic messages

---

## What's Changed

- feat: implement parallel file uploads using async runtime by @contributor in 011-implement-parallel-file-uploads-using-async-runtim
- fix: configure HTTP client timeouts to prevent denial of service by @contributor in 012-configure-http-client-timeouts-to-prevent-denial-o
- fix: replace panic calls with graceful error handling in production paths by @contributor in 013-replace-panic-with-graceful-error-handling-in-prod
- refactor: split client.rs to separate HTTP, file operations, and test concerns by @contributor in 009-split-client-rs-to-separate-concerns-510-lines
- refactor: replace panic and unwrap calls with proper error handling by @contributor in 010-replace-panic-calls-with-proper-error-handling


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
