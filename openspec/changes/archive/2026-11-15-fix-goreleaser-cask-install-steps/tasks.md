# Tasks

## 1. Reproduce the warning

- [x] 1.1 Upgrade local goreleaser with `brew upgrade goreleaser` and verify `goreleaser --version` reports 2.19 or newer
- [x] 1.2 Run `goreleaser check` on the unchanged `.goreleaser.yaml` and verify it prints the `homebrew_casks.hooks.post.install` deprecation

## 2. Update the cask block

- [x] 2.1 In `.goreleaser.yaml`, replace `hooks.post.install` with `hooks.post.install_steps` using `on_macos do` / `run "/usr/bin/xattr", args: ["-dr", "com.apple.quarantine", "{{ .StagedPath }}/paperless-ngx-uploader"]` / `end`, keep the existing `# Reason:` comment, and verify the old `install:` key is gone
- [x] 2.2 Set `homepage` to `https://github.com/oliverziegert/paperless-ngx-uploader` and `description` to `CLI tool for uploading documents to a Paperless-ngx instance`, and verify with `git diff` that no line outside the `homebrew_casks` block changed
- [x] 2.3 Run `goreleaser check` and verify it validates the file with no deprecation message

## 3. Integration checks

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` and verify all three pass

## Workflow follow-up

- Commit as `fix(release): Replace deprecated cask post-install hook with install_steps` and check `cog bump --auto --dry-run --skip-untracked` reports 0.4.1.
- Open the PR with `Refs #273` (not `Fixes`), so the issue stays open.
- Archive the change on the feature branch once CI is green, then commit and push the archive.
- After the 0.4.1 release: confirm the cask in `oliverziegert/homebrew-tap` contains a `postflight_steps` stanza and the new homepage and description.
- After the 0.4.1 release: run `brew install` on macOS and confirm Gatekeeper does not block the binary; then tick the boxes in issue 273 and close it.
