# Spec Delta

## Purpose

Defines the title that `upload` sends to Paperless-ngx with each document, so that every filename yields a title the server accepts.

## ADDED Requirements

### Requirement: Title is the filename cut to 64 characters
`upload` SHALL send as the document title the filename without its extension, cut to at most 64 characters. The limit SHALL count characters, not bytes, so that a cut never lands inside a character.

#### Scenario: Short name
- **WHEN** a file named `invoice.pdf` is uploaded
- **THEN** the title is `invoice`

#### Scenario: Long name
- **WHEN** a file whose name without extension has more than 64 characters is uploaded
- **THEN** the title is the first 64 characters of that name

#### Scenario: Multi-byte character on the limit
- **WHEN** a file named with 63 ASCII characters followed by `ß_rest.pdf` is uploaded
- **THEN** the upload does not fail because of the name
- **AND** the title is the 63 ASCII characters followed by `ß`
