# Spec Delta

## Purpose

Defines which files `upload --delete` removes from the `archive/` folder and how the upload summary reports them, so that old archived files are cleaned up and the reported count can be trusted.

## ADDED Requirements

### Requirement: Cleanup removes archived files older than the period
With `--delete`, `upload` SHALL remove each file in the `archive/` folder next to the uploaded files whose age is greater than the period given by `--period`. The age SHALL be the number of whole days since the file's modification time. Every other file SHALL be kept. A file that cannot be evaluated or removed SHALL NOT stop the cleanup of the remaining files.

#### Scenario: Old file
- **WHEN** the period is 31 days and `archive/` holds a file modified 40 days ago
- **THEN** the file is removed

#### Scenario: Recent file
- **WHEN** the period is 31 days and `archive/` holds a file modified 5 days ago
- **THEN** the file is kept

#### Scenario: File exactly as old as the period
- **WHEN** the period is 31 days and `archive/` holds a file modified 31 days ago
- **THEN** the file is kept

#### Scenario: File dated in the future
- **WHEN** `archive/` holds a file whose modification time lies in the future and a file modified 40 days ago
- **THEN** the future-dated file is kept
- **AND** the file modified 40 days ago is removed

### Requirement: Summary counts only removed files
The "Files deleted" count in the upload summary SHALL equal the number of files the cleanup removed in that run. Files that were kept and files whose removal failed SHALL NOT be counted.

#### Scenario: Old and recent files together
- **WHEN** the period is 31 days and `archive/` holds 2 files modified 40 days ago and 5 files modified 5 days ago
- **THEN** the summary reports 2 files deleted

#### Scenario: Nothing old enough
- **WHEN** the period is 31 days and every file in `archive/` was modified 5 days ago
- **THEN** the summary reports 0 files deleted

#### Scenario: File dated in the future
- **WHEN** the only file in `archive/` has a modification time in the future
- **THEN** the summary reports 0 files deleted
