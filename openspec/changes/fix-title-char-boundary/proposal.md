# Proposal

## Why

Uploading a file whose name has a multi-byte character (for example `ß`) across byte 64 panics, so the file is neither uploaded nor archived (GitHub issue #275). The title is cut at byte 64, and a byte cut inside a character is not allowed. Users run the tool unattended, so such a file fails on every run until someone renames it.

## What Changes

- The document title is cut after 64 characters instead of after 64 bytes. A cut can no longer land inside a character, so the upload no longer panics.
- Titles with non-ASCII characters may become longer than before: up to 64 characters instead of up to 64 bytes. Paperless-ngx stores up to 128 characters in the title, so every such title still fits.
- The limit stays at 64.
- Not changed: filenames that are not valid UTF-8, the handling of a panic inside an upload task, and letters built from a base letter plus a combining accent (a cut may separate the two).

Intended commit subject: `fix: Cut the document title by characters instead of bytes`. This is a patch bump, expected `v0.4.3`.

The change does not touch the HTTPS/localhost endpoint check or the token handling.

## Capabilities

### New Capabilities
- `document-title`: how the title sent with an uploaded document is derived from the filename.

### Modified Capabilities

None.

## Impact

- `src/cmd/client/helpers.rs`: `get_title_from_filename` and its comment; the `std::cmp::min` import becomes unused and is removed.
- `src/cmd/client/tests.rs`: two new title tests.
- Both callers in `src/cmd/client/http.rs` are fixed through the shared function and need no edit.
- No new crate. No change to `README.md` (it does not mention the title) or `CLAUDE.md` (it already says "64 characters").
