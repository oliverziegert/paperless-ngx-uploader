# Tasks

## 1. Status probe path

- [x] 1.1 Set `API_STATUS_PATH` in `src/cmd/client/helpers.rs` to `api/profile/` and update its doc comment and the `api_url` tests in the same file that assert on it; verify `cargo test api_url` passes
- [x] 1.2 In `Client::check_status` (`src/cmd/client/http.rs`) add a 404 branch whose error names `api/profile/`, says to check the endpoint, and says the Paperless-ngx version may be older than 2.1.0; update the docstring and the `/api/` comment; verify `cargo build` succeeds
- [x] 1.3 In `status_tests` (`src/cmd/client/tests.rs`) move every mock from `/api/` to `/api/profile/` (sub-path test: `/paperless/api/profile/`) and extend `test_check_status_not_found` to assert the error contains `api/profile/` and `2.1.0`; verify `cargo test status` passes
- [x] 1.4 Update `README.md` (the "Upgrading to 0.4.0" sentence "`status` appends `/api/`" and a note that `status` needs Paperless-ngx 2.1.0 or newer) and `CLAUDE.md` ("`status` appends `api/`"); verify `grep -n "appends" README.md CLAUDE.md` shows `api/profile/` for `status`

## 2. Integration checks

- [x] 2.1 Run `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test`; verify all pass and no file newly exceeds 500 lines (`src/cmd/client/tests.rs` is already at 796 and only gains two assertions)

## Workflow follow-up

- Commit with subject `fix: Probe api/profile/ in status instead of the API root` and run `cog bump --auto --dry-run --skip-untracked`; the expected version is `v0.4.1`.
- Open the pull request and wait for all CI checks.
- Archive this change on the branch right before the merge, then commit and push the archival.
- After the release, reply on #274 with the fixed version and close the issue.
