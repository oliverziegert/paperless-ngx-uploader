# api-request-urls Specification

## Purpose

Defines which Paperless-ngx API URL each command calls, so that every command derives its request URL from the same stored base URL.

## Requirements

### Requirement: Upload posts to the document upload API path
`upload` SHALL send each document as a POST request to the stored base URL followed by `/api/documents/post_document/`, with exactly one slash between base URL and path regardless of a trailing slash on the base URL.

#### Scenario: Base URL at the host root
- **WHEN** the stored endpoint is `https://host` and a file is uploaded
- **THEN** the request is `POST https://host/api/documents/post_document/`

#### Scenario: Base URL with trailing slash
- **WHEN** the stored endpoint is `https://host/` and a file is uploaded
- **THEN** the request is `POST https://host/api/documents/post_document/`

#### Scenario: Base URL with sub-path
- **WHEN** the stored endpoint is `https://host/paperless` and a file is uploaded
- **THEN** the request is `POST https://host/paperless/api/documents/post_document/`

#### Scenario: Server rejects the upload
- **WHEN** the upload API path answers with a non-200 status
- **THEN** the file is reported as failed with that status, as before

### Requirement: Status checks the profile API path
`status` SHALL send a GET request to the stored base URL followed by `/api/profile/`, composed by the same rule as the upload URL. A 404 answer SHALL be reported with a message that names the profile path and both possible causes: a wrong endpoint, or a Paperless-ngx version older than 2.1.0.

#### Scenario: Base URL at the host root
- **WHEN** the stored endpoint is `https://host` and the user runs `status`
- **THEN** the request is `GET https://host/api/profile/`

#### Scenario: Base URL with sub-path and trailing slash
- **WHEN** the stored endpoint is `https://host/paperless/` and the user runs `status`
- **THEN** the request is `GET https://host/paperless/api/profile/`

#### Scenario: Profile path is not found
- **WHEN** the profile path answers with 404
- **THEN** `status` fails
- **AND** the error names `api/profile/`, tells the user to check the endpoint, and says the Paperless-ngx version may be older than 2.1.0

#### Scenario: Other error status
- **WHEN** the profile path answers with a status other than 200, 401 or 404
- **THEN** `status` fails with an error that contains that status, as before

### Requirement: Upload and status agree on one endpoint value
For any stored base URL, `upload` and `status` SHALL address the same instance, so that a config with which `status` succeeds is also the config `upload` posts with.

#### Scenario: One config serves both commands
- **WHEN** the stored endpoint is `https://host` and the instance accepts the token
- **THEN** `status` reports success against `https://host/api/profile/`
- **AND** `upload` posts to `https://host/api/documents/post_document/` without any config change
