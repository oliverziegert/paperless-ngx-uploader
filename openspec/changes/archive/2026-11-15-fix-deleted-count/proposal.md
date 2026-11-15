# Proposal

## Why

With `--delete`, the upload summary reports every file in the `archive/` folder as deleted, including the files that were kept because they are not old enough (GitHub issue #276). No file is wrongly removed, but the "Files deleted" count and the debug line "File deleted successfully" are wrong. Users run the tool unattended and read the summary in logs, where a count of 7 deleted files after a run that deleted none looks like data loss.

## What Changes

- "Files deleted" counts only the files the cleanup actually removed. Files that are kept, and files whose deletion failed, are no longer counted.
- The debug line "File deleted successfully" is written only for a removed file.
- The rule that decides which files are removed does not change. It is written down for the first time: a file in `archive/` is removed when its age in whole days, measured from its modification time, is greater than `--period`.
- Not changed: the `archive/` folder is still read once per uploaded file, the dry-run path still does not evaluate deletions, and the oversized `src/cmd/client/tests.rs` is not split.

Intended commit subject: `fix: Count only removed files as deleted in the upload summary`. This is a patch bump, expected `v0.4.4`.

The change does not touch the HTTPS/localhost endpoint check or the token handling.

## Capabilities

### New Capabilities
- `archive-cleanup`: which files `upload --delete` removes from the `archive/` folder and how the summary reports them.

### Modified Capabilities

None.

## Impact

- `src/cmd/client/file_ops.rs`: `delete_expired_file` reports whether it removed the file; `delete_expired_files` counts and logs only removals. Docstrings updated.
- `src/cmd/client/file_ops_tests.rs`: new test file for the cleanup, which has no tests today.
- `src/cmd/client/mod.rs`: registers the new test module.
- The only caller, `Client::upload` in `src/cmd/client/http.rs`, takes the count as before and needs no edit.
- No new crate: the tests age files with `std::fs::File::set_modified` and use the existing `tempfile` dev-dependency.
- No change to `README.md` or `CLAUDE.md`: neither shows the summary or describes the count.
