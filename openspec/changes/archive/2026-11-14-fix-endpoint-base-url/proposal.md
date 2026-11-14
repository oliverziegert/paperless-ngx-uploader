# Proposal

## Why

`init` asks for the base URL of the Paperless-ngx instance (e.g. `https://paperless.example.com`), but `upload` posts to the stored endpoint exactly as it is, so the request hits the web UI and is rejected with `403 Forbidden` ([#269](https://github.com/oliverziegert/paperless-ngx-uploader/issues/269)). `status` appends `/api/` to the same value, so the two commands disagree about what the endpoint means: a base URL breaks `upload`, and the full upload URL (the workaround given in #269) breaks `status`.

## What Changes

- **BREAKING**: The stored endpoint is defined as the **base URL** of the instance. `upload` appends `/api/documents/post_document/` and `status` appends `/api/`. A config that stores the full upload URL no longer means "post exactly here".
- Endpoints ending in `/api/documents/post_document`, `/api/documents` or `/api` (with or without trailing slash) are reduced to the part before that suffix. Sub-paths such as `https://host/paperless` are kept.
- Existing config files are migrated automatically when loaded: the file is rewritten once and a notice names the old and new value. If the file cannot be written, the command continues with the corrected value and repeats the notice on each run.
- The automatic migration is announced as ending in **0.6.0**. From then on an old-style endpoint is an error that names the corrected value. This change specifies that behavior; removing the rewrite is a separate later change.
- `init` applies the same reduction to typed or `--endpoint` input and tells the user what was stored. This is permanent. Prompt and help texts say "base URL".
- Communication: a breaking-change section for 0.4.0 in `CHANGELOG.md`, an "Upgrading to 0.4.0" section in the README, a breaking commit subject, and a drafted (not posted) reply for #269.

Out of scope: hints on upload errors, a stronger `status` check, inspecting the stored token, a `config_version` field, stricter endpoint validation, backfilling `CHANGELOG.md` for 0.2.3–0.3.23.

## Capabilities

### New Capabilities

- `endpoint-configuration`: What the stored endpoint means, how endpoint values are reduced to a base URL at `init` and on config load, and how long old-style values are migrated automatically.
- `api-request-urls`: Which Paperless-ngx API URL each command calls, derived from the base URL.

### Modified Capabilities

None. The project has no specs yet.

## Impact

- **Code**: `src/cmd/client/http.rs` (upload and status URL), `src/cmd/client/helpers.rs` (shared URL helper), `src/cmd/config/file.rs` (migration on load), `src/main.rs` and `src/cmd/input.rs` (`init` input and texts), a new small module for the reduction rule.
- **Tests**: `src/cmd/client/tests.rs` currently configures `<server>/api/endpoint` as endpoint and mocks that exact path; these move to the real API paths. New tests for the reduction rule, the migration and `init`.
- **Users**: configs holding a base URL start working for `upload` with no action. Configs holding the workaround URL are rewritten on first run. Read-only configs keep working but print the notice on every run until edited.
- **Docs and release**: `README.md`, `CHANGELOG.md`, `CLAUDE.md` (endpoint description), the v0.4.0 GitHub release description (manual), the reply on #269 (manual, after the reporter confirms).
- **Dependencies**: none added.
