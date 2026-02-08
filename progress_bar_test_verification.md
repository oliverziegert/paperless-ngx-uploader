# Progress Bar Testing Verification

## Overview
This document outlines how to test the progress bar feature implemented for batch uploads in the Paperless-ngx Uploader CLI tool.

## Implementation Summary

The progress bar has been integrated into the `upload_files` method in `src/cmd/client/http.rs` using the `indicatif` crate (v0.17).

### Progress Bar Features
- **Visual Progress**: ASCII progress bar showing completion percentage
- **File Count**: Shows current file number and total files (e.g., "3/10")
- **Current File**: Displays the name of the file being uploaded
- **Timing Information**:
  - Elapsed time since upload started
  - Estimated Time to Arrival (ETA) for completion
- **Status Messages**: Shows "Uploaded: filename" or "Failed: filename" for each file
- **Completion Message**: Final message "Upload complete" when all files are processed

### Progress Bar Template
```
[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg} (ETA: {eta})
```

Example output:
```
[00:00:05] ===========>------------------- 3/10 Uploaded: test_file_3.pdf (ETA: 00:00:12)
```

## Test Setup

### Prerequisites
1. A running Paperless-ngx instance
2. Valid credentials configured in the uploader
3. Test files in `test_upload_folder/` directory

### Test Files Available
The repository includes a `test_upload_folder/` with multiple test files:
- 10 PDF files at root level (test_file_1.pdf through test_file_10.pdf)
- 3 additional PDF files in subfolders
- Total: 13 PDF files + 1 TXT file

## Manual Testing Procedures

### Test 1: Basic Progress Bar with Multiple Files
**Purpose**: Verify progress bar displays and updates correctly

**Command**:
```bash
cargo run -- upload --folder ./test_upload_folder --filter '.*\.pdf$'
```

**Expected Behavior**:
- ✅ Progress bar appears before upload starts
- ✅ Progress bar shows "0/10" initially
- ✅ Progress bar increments after each file upload (1/10, 2/10, etc.)
- ✅ Current filename is displayed in the message
- ✅ Elapsed time updates in real-time
- ✅ ETA is calculated and displayed
- ✅ Progress bar reaches "10/10" when complete
- ✅ Final message "Upload complete" is displayed
- ✅ Upload summary statistics are shown after progress bar completes

### Test 2: Progress Bar with Recursive Upload
**Purpose**: Verify progress bar works with recursive folder scanning

**Command**:
```bash
cargo run -- upload --folder ./test_upload_folder --recursive --filter '.*\.pdf$'
```

**Expected Behavior**:
- ✅ Progress bar shows "0/13" (all PDFs including subfolders)
- ✅ Progress bar updates correctly for nested files
- ✅ File paths show correctly for files in subfolders
- ✅ Progress bar reaches "13/13" when complete

### Test 3: Progress Bar with Failed Uploads
**Purpose**: Verify progress bar handles upload failures gracefully

**Setup**: Configure an invalid endpoint or use invalid credentials

**Expected Behavior**:
- ✅ Progress bar still increments even for failed uploads
- ✅ Failed uploads show "Failed: filename" message
- ✅ Progress bar continues to 100% even with failures
- ✅ Upload summary shows both successful and failed counts

### Test 4: Progress Bar Visual Appearance
**Purpose**: Verify the visual formatting is correct

**What to Check**:
- ✅ Progress bar uses cyan/blue colors
- ✅ Progress bar is approximately 40 characters wide
- ✅ Progress characters are "=>" for completed, "-" for remaining
- ✅ Time formats are precise (HH:MM:SS format)
- ✅ Progress bar doesn't interfere with log output
- ✅ Progress bar stays on one line (doesn't create multiple lines)

### Test 5: Progress Bar with Single File
**Purpose**: Verify progress bar works with edge case of single file

**Command**:
```bash
cargo run -- upload --file ./test_upload_folder/test_file_1.pdf
```

**Expected Behavior**:
- ✅ Progress bar shows "0/1" then "1/1"
- ✅ Progress bar completes quickly but displays correctly
- ✅ No visual glitches with rapid completion

## Verification Checklist

Before marking the feature complete, verify:

- [ ] Build compiles without errors: `cargo build`
- [ ] All unit tests pass: `cargo test`
- [ ] Clippy lints pass: `cargo clippy -- -D warnings`
- [ ] Progress bar displays when uploading multiple files
- [ ] Progress bar shows current file number / total files
- [ ] Progress bar shows file name being uploaded
- [ ] Progress bar shows elapsed time
- [ ] Progress bar shows ETA for completion
- [ ] Progress bar completes at 100% when all uploads finish
- [ ] Progress bar handles failed uploads gracefully
- [ ] Upload summary statistics still display correctly after progress bar
- [ ] Existing upload functionality continues to work
- [ ] No regression in error handling

## Known Limitations

1. **Dry-run Mode**: The progress bar is NOT shown in dry-run mode because the actual `upload_files` method is not called. Dry-run mode only simulates uploads without creating the concurrent task pool.

2. **Concurrent Uploads**: Files are uploaded in parallel (up to 10 concurrent uploads), so the progress bar may increment in non-sequential order.

3. **Log Output**: Individual file upload success messages from the logger may appear alongside the progress bar, which is expected behavior.

## Troubleshooting

### Progress Bar Not Showing
- Ensure you're NOT using `--dry-run` flag
- Verify you have multiple files to upload (progress bar is still shown for 1 file but completes quickly)
- Check that the folder exists and contains matching files

### Progress Bar Formatting Issues
- Terminal must support ANSI color codes
- Terminal width should be at least 80 characters for optimal display

### ETA Shows N/A or Incorrect Values
- ETA calculation requires at least one file to complete
- ETA becomes more accurate as more files are processed

## Success Criteria

✅ The progress bar feature is considered successfully implemented and tested when:

1. All automated tests pass (cargo test, cargo clippy)
2. Manual testing confirms all expected behaviors listed above
3. No regressions in existing upload functionality
4. Visual appearance matches the design specifications
5. Progress bar improves user experience without introducing bugs

## Testing Completed

**Date**: _________________
**Tester**: _________________
**Result**: [ ] PASS [ ] FAIL
**Notes**: _________________________________________________________________
________________________________________________________________________
________________________________________________________________________
