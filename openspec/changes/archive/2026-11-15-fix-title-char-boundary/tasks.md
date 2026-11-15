# Tasks

## 1. Title cut by characters

- [x] 1.1 In `get_title_from_filename` (`src/cmd/client/helpers.rs`) replace the byte cut with `name.chars().take(MAX_TITLE_LENGTH).collect()`, remove the unused `use std::cmp::min;`, and reword the comment and docstring: the limit counts characters, and 64 is the tool's own margin below the 128 characters Paperless-ngx stores; verify `cargo build` succeeds without warnings
- [x] 1.2 In `src/cmd/client/tests.rs` add two tests next to the existing title tests: a name of 63 ASCII characters followed by `ß_rest.pdf` expects the 63 characters plus `ß`, and a name of 70 `ä` plus `.pdf` expects 64 `ä`; verify `cargo test get_title_from_filename` passes, including the five existing title tests
- [x] 1.3 Confirm the docs need no edit: `grep -n -i "title" README.md CLAUDE.md` shows only the `CLAUDE.md` line that already says "64 characters"

## 2. Integration checks

- [x] 2.1 Run `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test`; verify all pass and no file newly exceeds 500 lines (`src/cmd/client/tests.rs` is already at 798 and gains two short tests)

## Workflow follow-up

- Commit with subject `fix: Cut the document title by characters instead of bytes` and run `cog bump --auto --dry-run --skip-untracked`; the expected version is `v0.4.3`.
- Open the pull request and wait for all CI checks.
- Archive this change on the branch right before the merge, then commit and push the archival.
- After the release, reply on #275 with the fixed version and close the issue.
