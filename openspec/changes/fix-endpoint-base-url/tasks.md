# Tasks

## 1. Endpoint reduction rule

- [x] 1.1 Create `src/cmd/endpoint.rs` with `normalize_endpoint` (path-based suffix match, design D1) and the sunset-version constant, register it in `src/cmd/mod.rs`, with Google-style docstrings; verify `cargo build` succeeds
- [x] 1.2 Add unit tests in the same module covering every scenario of "API path suffixes are reduced to the base URL" in `specs/endpoint-configuration/spec.md` (expected: three suffixes with and without slash; edge: sub-path, `/myapi`, host `https://api`, query string; failure: empty and unparsable input returned unchanged); verify `cargo test endpoint` passes

## 2. Request URL composition

- [x] 2.1 Add `api_url`, `API_STATUS_PATH` and `API_UPLOAD_PATH` to `src/cmd/client/helpers.rs` (design D2) with unit tests for no slash, trailing slash and sub-path; verify `cargo test api_url` passes
- [x] 2.2 Use `api_url` in `Client::check_status` and in `Client::upload_files` (the URL passed to `upload_file_task`) in `src/cmd/client/http.rs`, and update the affected docstrings; verify `grep -n "public_config.endpoint" src/cmd/client/http.rs` shows no direct use as a request URL
- [x] 2.3 Update `src/cmd/client/tests.rs`: both `create_test_config` helpers use `server.url()`, upload mocks become `POST /api/documents/post_document/`, status mocks become `GET /api/`, and add one sub-path test each for upload and status; verify `cargo test client` passes

## 3. Migration on config load

- [x] 3.1 In `PublicConfig::load()` (`src/cmd/config/file.rs`) reduce the loaded endpoint, save when it changed, and print the notice from design D4 with `eprintln!` plus `warn!`; a failed save prints the "could not update" line and does not return an error; add a `// Reason:` comment for the non-fatal save; verify `cargo build` succeeds
- [x] 3.2 Add tests in `src/cmd/config/file_tests.rs`: workaround URL is rewritten in the file and in memory; `allow_insecure: true` survives the rewrite; a base-URL file is byte-identical after load; a read-only file (`#[cfg(unix)]`) loads successfully with the reduced value and unchanged content; verify `cargo test file_tests` passes
- [x] 3.3 Run the built binary against a temp config holding `https://localhost:1/api/documents/post_document/` without `-v` (`cargo run -- --config <tmp> status`); verify stderr shows old value, new value and `0.6.0`, and the file now holds `https://localhost:1`

## 4. Init input and texts

- [x] 4.1 In `init()` (`src/main.rs`) reduce the endpoint before validation and print the "Stored endpoint as …" line when it changed (design D5); verify with `cargo run -- --config <tmp> init --endpoint https://localhost/api/documents/post_document/` that the message appears and the file holds `https://localhost`
- [x] 4.2 Change the `--endpoint` help text in `src/main.rs`, the prompt hint in `src/cmd/input.rs` and the `PublicConfig.endpoint` doc comment to say "base URL"; update `src/cmd/input_tests.rs` if it asserts on the old text; verify `cargo run -- init --help` shows the new wording and `cargo test input` passes
- [x] 4.3 Add tests for `init` covering a reduced `--endpoint`, an unchanged base URL, and an invalid endpoint that is still rejected with nothing saved; verify `cargo test init` passes

## 5. Documentation and announcement

- [x] 5.1 Add a hand-written `## 0.4.0` section at the top of `CHANGELOG.md` with a "⚠️ Breaking" block (endpoint is the base URL, automatic rewrite of existing configs, rewrite ends in 0.6.0, read-only configs must be edited by hand) and one line pointing to GitHub releases for 0.2.3–0.3.23; verify the section renders and names `0.6.0`
- [x] 5.2 Add an "Upgrading to 0.4.0" section to `README.md` with the same content and a before/after `config.yaml` example, and describe the endpoint as base URL in the init, configuration and security sections; verify `grep -n -i "endpoint" README.md` shows no remaining description that implies a full API URL
- [x] 5.3 Update the endpoint description and upload workflow in `CLAUDE.md` (base URL, paths appended per command, migration on load); verify the "Storage Backends" and "Upload Workflow" sections match the new behavior
- [x] 5.4 Review `openspec/changes/fix-endpoint-base-url/issue-269-reply.md` against the final behavior and wording of the notice; verify the quoted notice text matches the implementation. Do not post it

## 6. Integration and release checks

- [x] 6.1 Run `cargo fmt --check`, `cargo clippy -- -D warnings` (the CI command) and `cargo test`; verify all pass and no file added or grown by this change newly exceeds 500 lines
- [x] 6.2 Commit with subject `feat!: Store base URL as endpoint and append API paths per request` (design D6) and run `cog bump --auto --dry-run --skip-untracked` on the branch; verify the output is `v0.4.0`. If it is not, stop and raise it before merging, because README, changelog and notice all name 0.4.0 and 0.6.0

## Workflow follow-up

- After the release job has run, confirm the tag is `v0.4.0` and paste the breaking block from `CHANGELOG.md` into the GitHub release description (goreleaser only lists commit subjects).
- Post `issue-269-reply.md` on #269 only after the reporter has confirmed the workaround, then close the issue once the release is out.
- Open a follow-up change for 0.6.0 that replaces the automatic rewrite with the error specified in `endpoint-configuration`.
- Archive this change after review.
