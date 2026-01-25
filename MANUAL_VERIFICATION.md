# Manual Verification Guide: Parallel File Uploads

## Status: Ready for Manual Testing

### Build Status
✅ **Release binary built successfully** (`cargo build --release`)
- Binary location: `./target/release/paperless-ngx-uploader`
- Build time: 41.38s
- Warning: One unused method `upload_file` (expected - replaced by `upload_file_task` for parallel uploads)

### Test Environment Prepared
✅ **Test folder created** at `./test_upload_folder/`
- Contains 10 test PDF files (test_file_1.pdf through test_file_10.pdf)
- Each file contains sample content

---

## Manual Verification Steps

### Prerequisites
1. Access to a running Paperless-ngx server instance
2. Valid API token for authentication
3. Initialized configuration (run `./target/release/paperless-ngx-uploader init` if needed)

### Test 1: Multiple Files Upload Successfully

```bash
# Run the upload command
./target/release/paperless-ngx-uploader upload --folder ./test_upload_folder
```

**Expected Results:**
- All 10 files should upload without errors
- Console output should show "File <name> uploaded successfully" for each file
- Check Paperless-ngx web UI to confirm all 10 documents appear

**Verification:**
- [ ] All 10 files uploaded
- [ ] No error messages in console
- [ ] All documents visible in Paperless-ngx UI

---

### Test 2: Files Upload in Parallel

**Check Server Logs for Concurrent Requests:**

The parallel implementation uses `tokio::task::JoinSet` to spawn concurrent upload tasks. To verify parallelism:

1. Monitor Paperless-ngx server logs during upload:
   ```bash
   # On server, tail the logs
   docker logs -f paperless-ngx  # or check your server's log location
   ```

2. Run the upload:
   ```bash
   ./target/release/paperless-ngx-uploader upload --folder ./test_upload_folder -vv
   ```

**Expected Results:**
- Server logs should show multiple POST requests arriving concurrently (not sequentially)
- Timestamps should be close together (within seconds, not sequential)
- Client logs with `-vv` flag should show concurrent activity

**Code Evidence:**
- Implementation in `src/cmd/client/http.rs` lines 135-171
- Uses `tokio::task::JoinSet` to spawn parallel tasks
- Each file upload runs in its own task via `set.spawn()`

**Verification:**
- [ ] Server logs show concurrent requests (check timestamps)
- [ ] Upload completes faster than sequential (approx. 10x faster for 10 files)
- [ ] Multiple "File uploaded successfully" messages appear rapidly

---

### Test 3: Error Handling - Failed Upload Doesn't Stop Batch

**Create a test with a problematic file:**

```bash
# Add an invalid file to the test folder
echo "invalid content" > ./test_upload_folder/not_a_pdf.txt

# Run upload with filter that matches all files
./target/release/paperless-ngx-uploader upload --folder ./test_upload_folder --filter ".*"
```

**Or test with network errors:**
- Temporarily disconnect network during upload
- Use an invalid endpoint in config

**Expected Results:**
- Failed uploads should be logged with error message
- Other uploads should continue and complete successfully
- Process should not crash or abort

**Code Evidence:**
- Lines 155-168 in `src/cmd/client/http.rs` handle individual task failures
- Each task error is logged but doesn't stop the batch
- Failed files are not added to `files_archived` vector

**Verification:**
- [ ] Error logged for problematic file
- [ ] Other files continue uploading
- [ ] Application doesn't crash
- [ ] Successful uploads are still processed (archived if --archive flag used)

---

### Test 4: Logging Output

```bash
# Run with verbose logging
./target/release/paperless-ngx-uploader upload --folder ./test_upload_folder -vv
```

**Expected Results:**
- Debug messages show task spawning
- Individual file upload progress is logged
- No duplicate or missing log entries
- Log order may vary due to concurrent execution

**Verification:**
- [ ] All file uploads logged
- [ ] Debug messages visible with -vv flag
- [ ] No errors or warnings about logging

---

### Test 5: Stress Test (Optional)

Create more files to test higher concurrency:

```bash
# Create 50 test files
for i in {11..50}; do
  echo "Test document $i" > ./test_upload_folder/test_file_$i.pdf
done

# Upload with verbose logging
./target/release/paperless-ngx-uploader upload --folder ./test_upload_folder -vv
```

**Expected Results:**
- All 50 files upload successfully
- No memory issues or crashes
- Parallel execution visible in logs
- Server handles concurrent requests properly

---

## Implementation Details

### Parallel Upload Architecture

**File:** `src/cmd/client/http.rs`

**Key Changes:**
1. **Async Runtime** (line 74): `#[tokio::main]` on main function
2. **Async Client** (line 18): Changed from `reqwest::blocking::Client` to `reqwest::Client`
3. **Parallel Logic** (lines 135-171):
   - Creates `JoinSet` for managing concurrent tasks
   - Spawns one task per file
   - Each task calls `upload_file_task()` independently
   - Results collected as tasks complete

**Error Handling:**
- Individual task failures logged (line 162)
- Task join errors logged (line 165)
- Successful uploads collected in `files_archived`
- Failed uploads don't stop batch processing

**Concurrency:**
- All files spawn simultaneously (no artificial limit in current implementation)
- Tokio runtime handles task scheduling
- Network I/O is non-blocking

---

## Automated Test Coverage

✅ **All 49 unit tests pass** (completed in subtask-4-1)
- Async test conversion complete (`#[tokio::test]`)
- Mock server tests verify async behavior
- No regressions from async conversion

---

## Sign-Off Checklist

- [x] Release binary builds successfully
- [x] Unit tests pass (49/49)
- [x] Test files prepared
- [ ] **PENDING MANUAL VERIFICATION:** Multiple files upload successfully
- [ ] **PENDING MANUAL VERIFICATION:** Parallel uploads confirmed via server logs
- [ ] **PENDING MANUAL VERIFICATION:** Error handling works (failed upload doesn't stop batch)
- [ ] **PENDING MANUAL VERIFICATION:** Logging works correctly

---

## Notes for Reviewer

This implementation converts the sequential file upload process to parallel async uploads:

1. **Performance Improvement:** Expected ~10x speedup for batch uploads
2. **No Breaking Changes:** API remains the same for end users
3. **Backward Compatible:** Same CLI interface and configuration
4. **Error Handling Preserved:** Individual failures don't affect batch
5. **Test Coverage:** All existing tests updated and passing

The manual verification requires access to a live Paperless-ngx server, which is not available in the automated test environment.
