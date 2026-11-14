# endpoint-configuration Specification

## Purpose

Defines what the stored Paperless-ngx endpoint means and how endpoint values entered by the user or found in existing config files are reduced to that form.

## Requirements

### Requirement: Stored endpoint is the instance base URL
The endpoint stored in the config file SHALL be the base URL of the Paperless-ngx instance, including any sub-path the instance is served under, and SHALL NOT include an API path.

#### Scenario: Base URL at the host root
- **WHEN** the instance is reachable at `https://paperless.example.com`
- **THEN** the stored endpoint is `https://paperless.example.com`

#### Scenario: Instance served under a sub-path
- **WHEN** the instance is reachable at `https://example.com/paperless`
- **THEN** the stored endpoint is `https://example.com/paperless`

### Requirement: API path suffixes are reduced to the base URL
The system SHALL reduce an endpoint whose URL path ends in `/api/documents/post_document`, `/api/documents` or `/api`, with or without a trailing slash, to the part before that suffix. The comparison SHALL apply to whole path segments of the URL path only, never to the host. Any other endpoint SHALL be left unchanged.

#### Scenario: Full upload URL
- **WHEN** the endpoint is `https://host/api/documents/post_document/`
- **THEN** the reduced endpoint is `https://host`

#### Scenario: API root without trailing slash
- **WHEN** the endpoint is `https://host/api`
- **THEN** the reduced endpoint is `https://host`

#### Scenario: Sub-path is preserved
- **WHEN** the endpoint is `https://host/paperless/api/documents/`
- **THEN** the reduced endpoint is `https://host/paperless`

#### Scenario: Base URL is left unchanged
- **WHEN** the endpoint is `https://host/paperless`
- **THEN** the reduced endpoint is `https://host/paperless`

#### Scenario: Suffix must be a whole path segment
- **WHEN** the endpoint is `https://host/myapi`
- **THEN** the endpoint is left unchanged

#### Scenario: Host named like the suffix
- **WHEN** the endpoint is `https://api`
- **THEN** the endpoint is left unchanged

### Requirement: Init stores the reduced endpoint and says so
`init` SHALL apply the reduction to the endpoint given at the prompt or with `--endpoint` before saving it. When the reduction changed the value, `init` SHALL tell the user which value was stored. This behavior has no end date.

#### Scenario: Upload URL given with --endpoint
- **WHEN** the user runs `init --endpoint https://host/api/documents/post_document/`
- **THEN** the config file contains `https://host` as endpoint
- **AND** the output states that `https://host` was stored and that API paths are added automatically

#### Scenario: Base URL given at the prompt
- **WHEN** the user enters `https://host` at the endpoint prompt
- **THEN** the config file contains `https://host` as endpoint
- **AND** no reduction message is shown

#### Scenario: Invalid endpoint is still rejected
- **WHEN** the user runs `init --endpoint host/api/`
- **THEN** `init` fails with the existing invalid-URL error and nothing is saved

### Requirement: Existing config files are migrated on load
From version 0.4.0 until version 0.6.0, any command that loads the config SHALL apply the reduction to the stored endpoint. When the value changes, the system SHALL rewrite the config file with the reduced endpoint, keep all other settings, and use the reduced endpoint for the current run.

#### Scenario: Config holds the workaround URL
- **WHEN** the config file stores `https://host/api/documents/post_document/` and the user runs `upload`
- **THEN** the config file stores `https://host` afterwards
- **AND** the upload of that run uses `https://host` as base URL

#### Scenario: Other settings survive the rewrite
- **WHEN** the config file stores `allow_insecure: true` and an endpoint ending in `/api/`
- **THEN** the rewritten file still stores `allow_insecure: true`

#### Scenario: Config already holds a base URL
- **WHEN** the config file stores `https://host`
- **THEN** the file is not rewritten and no migration notice is shown

### Requirement: Migration notice is visible and names its end
When a load changes the stored endpoint, the system SHALL print a notice at default verbosity, without any `-v` flag, that names the old value, the new value, and version 0.6.0 as the release in which automatic rewriting ends.

#### Scenario: Notice on migration
- **WHEN** a command migrates `https://host/api/` to `https://host` and no `-v` flag is given
- **THEN** stderr contains both values and the text `0.6.0`

### Requirement: A failed rewrite does not fail the command
When the config file cannot be rewritten during migration, the system SHALL report that the file could not be updated, continue the command with the reduced endpoint, and repeat the notice on every later run until the file holds a base URL.

#### Scenario: Read-only config file
- **WHEN** the config file stores `https://host/api/documents/post_document/` and is not writable
- **THEN** the command proceeds using `https://host`
- **AND** the output states that the config file could not be updated
- **AND** the file content is unchanged

### Requirement: Old-style endpoints are rejected after the migration period
From version 0.6.0, the system SHALL NOT rewrite the config file automatically. Loading a config whose endpoint ends in one of the API path suffixes SHALL fail with an error that names the stored value and the base URL to replace it with.

#### Scenario: Old-style endpoint in 0.6.0 or later
- **WHEN** version 0.6.0 or later loads a config storing `https://host/api/documents/post_document/`
- **THEN** the command fails before any request is sent
- **AND** the error names `https://host` as the value to store
- **AND** the config file is unchanged
