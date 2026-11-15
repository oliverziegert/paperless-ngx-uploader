# Proposal

## Why

The release build prints a goreleaser deprecation warning for `homebrew_casks.hooks.post.install` (GitHub issue 273). Homebrew replaced the `postflight` stanza with `postflight_steps`, and goreleaser will drop the old key in its next major version, at which point the cask would lose the step that keeps Gatekeeper from blocking the un-notarized binary.

The same cask block also still carries two template leftovers: the description `Example rust release.` and a homepage that points at the tap repository.

## What Changes

- Replace `hooks.post.install` in `.goreleaser.yaml` with `hooks.post.install_steps`, rewritten in Homebrew's declarative steps DSL (`on_macos` / `run` / `{{ .StagedPath }}`). The step still removes `com.apple.quarantine` from the installed binary on macOS.
- Set the cask `homepage` to `https://github.com/oliverziegert/paperless-ngx-uploader`.
- Set the cask `description` to `CLI tool for uploading documents to a Paperless-ngx instance`.

Nothing changes in the Rust code, the CLI, the config file or the token handling. The HTTPS/localhost endpoint check is untouched.

**Commit subject:** `fix(release): Replace deprecated cask post-install hook with install_steps`

This causes a **patch bump** (0.4.0 → 0.4.1). The `fix` type is deliberate: only a released version regenerates the cask in `homebrew-tap`, so a `ci` or `build` commit would leave the warning and the wrong cask metadata in place until some unrelated release.

## Capabilities

### New Capabilities

- `homebrew-distribution`: what a user gets when installing the tool through the Homebrew cask, starting with the guarantee that the binary runs on macOS without a Gatekeeper block.

### Modified Capabilities

None.

## Impact

- `.goreleaser.yaml`: the `homebrew_casks` block only.
- `oliverziegert/homebrew-tap`: the next release writes a cask with a `postflight_steps` stanza and the new homepage and description.
- No crate is added. No workflow file changes.
- Local tooling: verification needs goreleaser 2.19 or newer (installed: 2.15.4).
- Out of scope: `Cargo.toml` metadata, the GitHub repository description, a `goreleaser check` job in PR CI.
