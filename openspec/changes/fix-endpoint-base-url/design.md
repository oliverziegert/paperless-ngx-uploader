# Design

## Context

See `proposal.md` for motivation. Observed state of the code:

- `Client::upload_files` clones `cfg.public_config.endpoint` and `upload_file_task` posts to it unchanged (`src/cmd/client/http.rs:337`, `:424`).
- `Client::check_status` builds `format!("{}/api/", endpoint.trim_end_matches('/'))` (`src/cmd/client/http.rs:181`).
- `PublicConfig::load()` (`src/cmd/config/file.rs`) is the single place every non-`init` command reads the YAML file. `init` starts from `Config::default()` and never loads.
- `init()` in `src/main.rs` receives the endpoint from `--endpoint` or `get_endpoint_by_prompt()` and validates it afterwards.
- `setup_logging` maps no `-v` flag to `LevelFilter::Error`. A `warn!` is invisible by default. User-facing text in this codebase is printed with `println!`/`eprintln!` (see `src/cmd/input.rs`).
- `src/cmd/url_validator.rs` (632 lines) and `src/cmd/client/tests.rs` (753 lines) already exceed the project's 500-line limit.
- The client tests configure `<server>/api/endpoint` as endpoint and mock `POST /api/endpoint` and `GET /api/endpoint/api/`.
- Release versions come from `cog bump --auto`. Tested locally with cocogitto 6.5.0 on a tag `v0.3.23`: `fix!:` yields `v0.3.24`, a `BREAKING CHANGE:` footer on `fix:` yields `v0.3.24`, `feat!:` and `feat:` yield `v0.4.0`.

## Goals / Non-Goals

**Goals:**

- One rule for reducing an endpoint, used by `init` and by config load.
- One rule for composing request URLs, used by `upload` and `status`.
- A migration that needs no user action and whose end is announced from the first release.

**Non-Goals:**

- Removing the migration (a later change for 0.6.0).
- Restructuring the oversized `url_validator.rs` or `client/tests.rs`.
- Changing what `status` verifies.

## Decisions

### D1: Reduction rule lives in a new module `src/cmd/endpoint.rs`

A function `normalize_endpoint(endpoint: &str) -> String` plus its tests, and a constant for the sunset version (`"0.6.0"`) used by the notice text.

- Why not `url_validator.rs`: it is already over the 500-line limit.
- The suffix is matched against the **URL path**, not the raw string. A raw `ends_with("/api")` would turn `https://api` into `https:/`. The function parses with the `url` crate (already a dependency), checks the path with trailing slashes removed for the three suffixes at a segment boundary, longest first, and cuts that many characters off the original string. A value that does not parse (including the empty default endpoint) is returned unchanged; format errors stay the validator's job.
- Single pass: `https://host/api/api/` becomes `https://host/api`. Repeating until stable was considered and dropped; nobody has such a URL and a fixed point loop hides typos.
- Endpoints with a query or fragment are returned unchanged. The validator accepts them today and tightening that is out of scope.

### D2: Request URLs are composed by one helper in `src/cmd/client/helpers.rs`

`api_url(endpoint: &str, path: &str) -> String` as `format!("{}/{}", endpoint.trim_end_matches('/'), path)`, with two constants: `API_STATUS_PATH = "api/"` and `API_UPLOAD_PATH = "api/documents/post_document/"`. `check_status` and `upload_files` call it; `upload_file_task` keeps receiving a ready URL.

- Why string formatting over `Url::join`: `join` drops the last path segment of a base without trailing slash (`https://host/paperless` + `api/` gives `https://host/api/`), which breaks sub-path installs. The existing `check_status` line already uses the format approach.
- The client does not normalize again. It trusts the config, which is reduced on load and at `init`.

### D3: Migration happens inside `PublicConfig::load()`

After parsing, the endpoint is passed through `normalize_endpoint`. If it changed: set the reduced value in memory, try `save()`, print the notice.

- Why here: it is the one read path for `upload` and `status`, so no command can skip it.
- Save failure maps to a second notice line, not to an error return.
- The missing-file branch (defaults written on first run) is untouched; an empty endpoint is never changed.

Alternatives rejected: a separate `migrate` command (nobody runs it before hitting the error); migrating in `Config::load()` in `mod.rs` (works, but `PublicConfig` owns the file and its `save()`).

### D4: The notice is printed with `eprintln!`, plus a `warn!` for log consistency

Agreed during planning was "a `warn!` line". Because the default log level is `Error`, that alone would be silent for almost every user, which defeats the announcement. The notice therefore goes to stderr unconditionally, in the same style as the existing `eprintln!` messages. Text:

```
Notice: the configured endpoint '<old>' contains an API path.
The endpoint must be the base URL of your Paperless-ngx instance; using '<new>'.
<path> was updated.            | Could not update <path>; please change the endpoint manually.
Automatic correction will be removed in version 0.6.0.
```

`init` prints its permanent one-liner with `println!`: `Stored endpoint as '<new>' (API paths are added automatically).`

### D5: `init` normalizes in `init()`, before validation

One call after the endpoint is obtained covers both the `--endpoint` and the prompt branch. Normalizing inside `get_endpoint_by_prompt()` as well would duplicate the rule's call site for no gain.

### D6: The release commit is `feat!:`, not `fix!:`

Agreed was the subject `fix!: Store base URL as endpoint and append API paths per request` and version 0.4.0. The two conflict: cocogitto maps `fix!:` on a 0.x version to a patch bump. The version is the part users were promised, so the subject becomes `feat!: Store base URL as endpoint and append API paths per request`. The `!` and the wording still reach the goreleaser-generated notes.

### D7: Tests move to real API paths

In `src/cmd/client/tests.rs` both `create_test_config` helpers set the endpoint to `server.url()`, upload mocks become `POST /api/documents/post_document/`, status mocks become `GET /api/`. One added test per command uses `<server>/paperless` to pin the sub-path case. New tests are few; the file is not split in this change.

Migration tests go in `src/cmd/config/file_tests.rs` (61 lines) using `tempfile`, as the existing tests there do.

## Risks / Trade-offs

- [Read-only config prints the notice on every run, e.g. in cron mail] → Intended; the notice names the manual fix. Documented in the README upgrade section.
- [A user's instance really lives under a path ending in `/api`, e.g. a reverse proxy at `https://host/api`] → Such a config was already broken for `status` only if it had no further API suffix; after the change it is reduced to `https://host` and both commands fail. Accepted as vanishingly rare; the notice shows exactly what was changed so the cause is visible.
- [`0.6.0` is announced but version numbers are produced by commit types] → The removal change must be the commit that produces `0.6.0`. If no `0.5.0` has been released by then, a `feat!:` removal would become `0.5.0`. The removal change has to check `cog bump --auto --dry-run` first and wait or adjust the announcement.
- [CI installs the latest cocogitto, the local test used 6.5.0] → A task runs the dry-run on the branch before merge and compares with the CI output of the release job.
- [Read-only-file test passes trivially when tests run as root] → Mark the test `#[cfg(unix)]` and assert the file content is unchanged, which still holds; note the limitation in the test's doc comment.
- [The reporter of #269 has not confirmed the diagnosis] → The code defect exists regardless: `init` documents a base URL and `upload` does not append the API path. Only the issue reply waits for confirmation.

## Migration Plan

1. Merge with the `feat!:` subject; the release job tags `v0.4.0`.
2. Paste the breaking block from `CHANGELOG.md` into the `v0.4.0` GitHub release description.
3. After the reporter confirms, post `issue-269-reply.md` on #269.
4. Later change, released as `0.6.0`: replace the rewrite in `PublicConfig::load()` with the error specified in `endpoint-configuration`; keep `normalize_endpoint` for `init` and for computing the suggested value.

Rollback: revert the commit. Configs already migrated hold a base URL, which the previous version handles for `status` but not for `upload`; users who relied on the workaround URL would need to restore it by hand.
