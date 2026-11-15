# Proposal

## Why

In 0.4.0, `status` fails with `406 Not Acceptable` against a current Paperless-ngx instance although endpoint and token are valid ([#274](https://github.com/oliverziegert/paperless-ngx-uploader/issues/274)). `status` sends `GET <base URL>/api/`; current Paperless-ngx redirects the API root to the Swagger view, which only serves HTML and rejects the client's `Accept: application/json` header.

## What Changes

- `status` sends `GET <base URL>/api/profile/` instead of `GET <base URL>/api/`. The profile path returns JSON to any valid token and needs no further permission.
- A `404 Not Found` answer gets its own message: the profile path was not found, so the user should check the endpoint, or the Paperless-ngx version is older than 2.1.0 (the route is absent in v2.0.1 and present from v2.1.0).
- Unchanged: 200 and 401 behave as today, every other status keeps the generic error, the response body is ignored, and the success output stays the same.
- Not included: a fallback probe for instances older than 2.1.0, and showing the account name from the profile. `/api/health/` was considered, but Paperless-ngx has no such route; `/api/status/` only answers admin users.

Intended commit subject: `fix: Probe api/profile/ in status instead of the API root`. It is a `fix` without `!`, so the release is a patch bump to 0.4.1.

The change does not touch the HTTPS/localhost endpoint check or how the token is handled; the request carries the same `Authorization` header as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `api-request-urls`: the requirement "Status checks the API root path" changes to the profile path and gains a 404 scenario; the scenario under "Upload and status agree on one endpoint value" names the new status URL.

## Impact

- `src/cmd/client/helpers.rs`: value and doc comment of `API_STATUS_PATH`, and the `api_url` tests that assert on it.
- `src/cmd/client/http.rs`: one new branch in `Client::check_status` and its docstring.
- `src/cmd/client/tests.rs`: the status mocks move to `/api/profile/`; the 404 test asserts the hint.
- `README.md` and `CLAUDE.md`: the places that say `status` appends `/api/`.
- No new crate, no config change. `status` now needs Paperless-ngx 2.1.0 or newer; `upload` is unaffected.
