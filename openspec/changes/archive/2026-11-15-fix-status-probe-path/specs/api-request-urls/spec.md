# Spec Delta

## RENAMED Requirements

- FROM: `### Requirement: Status checks the API root path`
- TO: `### Requirement: Status checks the profile API path`

## MODIFIED Requirements

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
