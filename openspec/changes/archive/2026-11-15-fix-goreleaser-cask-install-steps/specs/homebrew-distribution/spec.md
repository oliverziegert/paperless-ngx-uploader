# Spec Delta

## Purpose

Describes what a user gets when installing the tool through the Homebrew cask published to the project's tap.

## ADDED Requirements

### Requirement: Cask install leaves a runnable binary on macOS
The release binaries are not Apple-notarized. Installing the Homebrew cask on macOS SHALL remove the `com.apple.quarantine` attribute from the installed binary, so that Gatekeeper does not block it. The release configuration SHALL express this step without options that the release tooling reports as deprecated.

#### Scenario: Install on macOS
- **WHEN** a user installs the cask on macOS
- **THEN** the installed `paperless-ngx-uploader` binary carries no `com.apple.quarantine` attribute
- **AND** running it is not blocked by Gatekeeper

#### Scenario: Release configuration is free of deprecations
- **WHEN** the release configuration is validated with the current release tooling
- **THEN** validation succeeds and reports no deprecation for the cask install step
