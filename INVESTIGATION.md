# Investigation: Replace `inquire` with a lighter-weight alternative

## 1. Current Usage Analysis

`inquire` is declared in `Cargo.toml` (v 0.9.1; resolved to 0.9.2 in Cargo.lock) and
imported in exactly one file — `src/cmd/input.rs`.  Two types are used:

| inquire type | Where | Features exercised |
|---|---|---|
| `Text` | `get_endpoint_by_prompt()` | `.with_placeholder("https://paperless.example.com")` |
| | | `.with_help_message("Use HTTPS …")` |
| | | `.prompt()` → reads one line |
| `Password` | `get_token_by_prompt()` | `.prompt()` → reads one line with echo suppressed |

### What is *not* used

- Validation / custom validators
- Select / multi-select / fuzzy-select prompts
- Theming / colour customisation
- Confirm prompts
- Any programmatic cursor / readline editing
- History or tab-completion

The two prompts need only five primitive capabilities:

1. Print a label and an inline hint (placeholder text)
2. Print a help / advisory message
3. Read a single line of text from stdin
4. Read a single line of text from stdin **with echo suppressed**
5. Return the text as a `String`

---

## 2. Dependency-tree audit — what `inquire` actually pulls in

`inquire 0.9.2` declares six direct dependencies.  Tracing the full transitive closure
via `Cargo.lock` yields the following crates that are **unique to the inquire subtree**
(i.e. no other non-dev dependency in this project requires them):

| # | Crate | Role |
|---|---|---|
| 1 | inquire | top-level prompt library |
| 2 | crossterm | cross-platform terminal control |
| 3 | crossterm_winapi | Windows terminal API shim |
| 4 | dyn-clone | runtime trait-object cloning |
| 5 | fuzzy-matcher | fuzzy string matching (unused feature) |
| 6 | thread_local | thread-local storage (dep of fuzzy-matcher) |
| 7 | derive_more | procedural macro helpers |
| 8 | convert_case | string-case conversion (dep of derive_more) |
| 9 | document-features | doc-generation macro (dep of crossterm) |
| 10 | signal-hook | Unix signal handling (dep of crossterm) |
| 11 | signal-hook-mio | mio integration for signal-hook |
| 12 | unicode-segmentation | grapheme cluster iteration |
| 13 | unicode-width | character display-width calculation |
| 14 | winapi | legacy Windows API bindings |

**14 crates would be removed** when `inquire` is dropped.

Several crossterm transitive deps (`mio`, `parking_lot`, `signal-hook-registry`,
`rustix`, `bitflags`, `libc`) are **already required by `tokio`** (or `tempfile` for
dev builds), so they will remain in the lock-file regardless.

---

## 3. Alternatives considered

### 3.1  `dialoguer` 0.12.0

| Aspect | Detail |
|---|---|
| Latest version | 0.12.0 (Aug 2025) |
| Direct deps | `console` ^0.16, `shell-words` ^1.1 (+ optional `fuzzy-matcher`, `tempfile`, `zeroize`) |
| API similarity | `Input::new()` / `Password::new()` — nearly identical builder pattern to inquire |
| Terminal stack | Still wraps `console`, which itself wraps `term` / platform APIs |

**Trade-offs:**
- *Pro:* Familiar API; simpler than raw stdin for placeholder handling.
- *Con:* Introduces `console` + its own transitive tree; the net saving over inquire is
  modest because a terminal-handling crate is still required.  The swap is one prompt
  library for another — the architectural motivation (remove the terminal-handling stack
  entirely) is not addressed.

### 3.2  Manual `std::io` + `rpassword`

| Aspect | Detail |
|---|---|
| `rpassword` latest | 7.4.0 (Apr 2025) |
| `rpassword` direct deps | `libc` (Unix), `rtoolbox`, `windows-sys` (Windows) |
| `rtoolbox` direct deps | `libc` (Unix), `windows-sys` (Windows) |
| New crates added | **2** — `rpassword` and `rtoolbox` |
| Already-present deps | `libc` and `windows-sys` (multiple versions) are already in the lock-file |
| Text-input mechanism | `std::io::stdin().read_line()` — zero new dependencies |

**Trade-offs:**
- *Pro:* Adds only 2 crates with zero new transitive deps beyond what is already locked.
  Removes 14 crates from the inquire subtree.  Net change: **−12 crates**.
- *Pro:* `rpassword` is a single-purpose, well-maintained crate (stable API since v5).
  Cross-platform echo suppression (Unix `termios`, Windows `SetConsoleMode`) is its sole
  job.
- *Pro:* Text prompts use `std::io` — part of the standard library, fully understood, no
  surprises.
- *Con:* No readline-style cursor editing (arrow keys to move within the line).  For a
  URL and an opaque token this is an acceptable trade-off.
- *Con:* Placeholder text must be communicated as a visual hint rather than an editable
  pre-filled value.  A `print!()` label like `Endpoint [https://paperless.example.com]: `
  achieves the same UX intent.

### 3.3  Pure manual (stdin + manual termios / Windows API)

Writing echo-suppression from scratch avoids even `rpassword`, but duplicates its
cross-platform logic and adds maintenance burden.  Rejected in favour of `rpassword`.

---

## 4. Dependency-count summary

| Scenario | Crates removed | Crates added | Net change |
|---|---|---|---|
| Replace with `dialoguer` | 14 (inquire tree) | ~4–6 (dialoguer + console + transitive) | −8 to −10 |
| Replace with `std::io` + `rpassword` | 14 (inquire tree) | 2 (rpassword + rtoolbox) | **−12** |
| Pure manual (no rpassword) | 14 (inquire tree) | 0 | −14 |

---

## 5. Recommendation

**Replace `inquire` with `std::io::stdin` for text input and `rpassword` for password
input.**

### Rationale

1. **Maximum dependency reduction.** −12 net crates is the best achievable outcome that
   does not require reimplementing platform-specific echo suppression.
2. **Scope matches need.** The project uses exactly two primitive I/O operations.  A
   focused single-purpose crate (`rpassword`) and the standard library cover both without
   pulling in a terminal-control framework.
3. **Minimal code change.** Only `src/cmd/input.rs` changes; the two public function
   signatures (`pub fn get_endpoint_by_prompt() -> Result<String, Box<dyn Error>>` and
   `pub fn get_token_by_prompt() -> Result<String, Box<dyn Error>>`) remain identical.
4. **`rpassword` is battle-tested.** Stable API, active maintenance, widely adopted in
   the Rust ecosystem for exactly this use-case.

---

## 6. Implementation notes

### `Cargo.toml`
- Remove: `inquire = { version = "0.9.1" }`
- Add: `rpassword = "7"` (latest stable is 7.4.0)

### `src/cmd/input.rs` — `get_endpoint_by_prompt()`
```
1. print the help message via eprintln! (or print! to stdout).
2. print! the prompt with the placeholder hint:
       "Endpoint [https://paperless.example.com]: "
3. Flush stdout (io::stdout().flush()?).
4. Read a line via std::io::stdin().read_line(&mut buf).
5. Trim whitespace.  If the result is empty, use the placeholder value.
6. Return the string.
```

### `src/cmd/input.rs` — `get_token_by_prompt()`
```
1. Call rpassword::prompt_password("Token: ").
2. Return the resulting String.
```

Both functions must preserve:
- Existing doc-comments (update "interactive prompt" wording where needed).
- `debug!` / `info!` / `error!` logging calls.
- `?`-based error propagation (no `unwrap` / `expect`).
