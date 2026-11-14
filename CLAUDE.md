# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🔄 Project Awareness & Context
- **Use consistent naming conventions, file structure, and architecture patterns**.

## 🧱 Code Structure & Modularity
- **Never create a file longer than 500 lines of code.** If a file approaches this limit, refactor by splitting it into modules or helper files.
- **Organize code into clearly separated modules**, grouped by feature or responsibility.
- **Use clear, consistent imports** (prefer relative imports within packages).

## 🧪 Testing & Reliability
- **Always create unit tests for new features** (functions, classes, routes, etc).
- **After updating any logic**, check whether existing unit tests need to be updated. If so, do it.
- **Tests should follwo Rust best pracices**.
    - Include at least:
        - 1 test for expected use
        - 1 edge case
        - 1 failure case

## 📎 Style & Conventions
- **Use Rust** as the primary language.
- Write **docstrings for every function** using the Google style:
  ```rust
  /// Brief summary.
  ///
  /// # Arguments
  ///
  /// * `param1` - Description.
  ///
  /// # Returns
  ///
  /// Description.
  fn new(name: &str) -> Person {
  ```

## 📚 Documentation & Explainability
- **Comment non-obvious code** and ensure everything is understandable to a mid-level developer.
- When writing complex logic, **add an inline `# Reason:` comment** explaining the why, not just the what.

## 🧠 AI Behavior Rules
- **Never assume missing context. Ask questions if uncertain.**
- **Never hallucinate libraries or functions** – only use known, verified Python packages.
- **Always confirm file paths and module names** exist before referencing them in code or tests.
- **Never delete or overwrite existing code** unless explicitly instructed to.

Always use context7 when I need code generation, setup or configuration steps, or
library/API documentation. This means you should automatically use the Context7 MCP
tools to resolve library id and get library docs without me having to explicitly ask.

## Project Overview

Paperless-ngx-uploader is a Rust CLI tool for uploading documents to a Paperless-ngx instance. It supports batch uploads, file filtering, archiving, and automated cleanup of old files.

## Build and Test Commands

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run with logging (for debugging)
RUST_LOG=debug cargo run -- [command]
```

## Architecture

### Configuration System

The application uses a **split configuration approach** to separate sensitive and non-sensitive data:

#### Storage Backends

- **Public Config** (YAML file): Non-sensitive configuration stored at `~/.config/paperless-ngx-uploader/config.yaml`
  - Contains: Paperless-ngx endpoint, which is the **base URL** of the instance (e.g. `https://paperless.example.com`, sub-paths allowed, no API path)
  - Managed by: `PublicConfig` struct in `src/cmd/config/file.rs`
  - Uses: `serde-saphyr` crate for YAML serialization

- **Private Config** (OS Keyring): Sensitive credentials stored in platform-native secure storage
  - Contains: Authentication token
  - Managed by: `PrivateConfig` struct in `src/cmd/config/keyring.rs`
  - Platform support:
    - macOS: Keychain
    - Linux: Secret Service
    - Windows: Credential Manager

#### Configuration Architecture

- **Config struct** (`src/cmd/config/models.rs`): Coordinates both storage backends
- **HandleConfig trait** (`src/cmd/config/mod.rs`): Common interface for `load()`, `save()`, and `delete()` operations
- **Endpoint reduction** (`src/cmd/endpoint.rs`): `normalize_endpoint` cuts a trailing `/api/documents/post_document`, `/api/documents` or `/api` off an endpoint
  - `init` applies it to the given endpoint and reports the stored value
  - `PublicConfig::load()` applies it to existing files (migration): the file is rewritten and a notice printed to stderr; a failed rewrite is not an error. The rewrite is announced to be removed in 0.6.0 (`MIGRATION_SUNSET_VERSION`)
- **Coordination methods**:
  - `Config::load()`: Loads from both YAML file and keyring
  - `Config::save()`: Saves to both YAML file and keyring
  - `Config::delete()`: Removes both YAML file and keyring entry

#### Testing Configuration

Configuration tests use the keyring-core mock credential store to prevent deadlocks:
- Mock store installed once via `keyring_core::set_default_store(keyring_core::mock::Store::new()...)` in `setup_logger()` (`src/cmd/config/tests.rs`)
- Tests touching the shared `(APP_NAME, "token")` entry are serialized via `KEYRING_LOCK` since they would otherwise race on the same mock credential
- One `#[ignore]`d smoke test exercises the real OS keyring: `cargo test smoke_real_store_roundtrip -- --ignored`

### Command Structure

The CLI uses clap for argument parsing with two main commands:
- **init**: Initializes configuration (endpoint and token)
- **upload**: Uploads files/folders to Paperless-ngx

### Upload Workflow

1. **Aggregation** (`aggregate_files`): Collects files from `--file` or `--folder` arguments, applying regex filter
2. **Upload** (`upload_files`): Uploads each file via multipart/form-data POST request to `<base URL>/api/documents/post_document/`
3. **Archive** (optional): Moves uploaded files to an `archive/` subfolder
4. **Cleanup** (optional): Deletes archived files older than specified period (default 31 days)

Request URLs are composed from the stored base URL by `api_url` (`src/cmd/client/helpers.rs`): `upload` appends `api/documents/post_document/`, `status` appends `api/`.

The client uses reqwest with blocking API and sets the Authorization header to `Token <token>`.

### File Title Generation

File titles sent to Paperless-ngx are derived from filenames (without extension) and truncated to 64 characters maximum (`MAX_TITLE_LENGTH`).

## Key Implementation Details

- **HTTP Client**: Created in `Client::new()` with Authorization header pre-configured
- **Error Handling**: Uses custom `CmdError` enum with specific, contextual errors (ConfigFile*, Keyring*, etc.)
- **Logging**: Uses `env_logger` with configurable verbosity via `-v` flag
- **Testing**: Unit tests use `mockito` for HTTP mocking, `tempfile` for isolated test environments
