# AGENTS.md — Concord (Discord TUI)

This file is the **only** repo-local instruction source for OpenCode / Kilo
sessions. Keep it compact; every line must answer "would an agent miss this
without help?"

---

## 1 · Project layout (single crate, no workspace)

| Path | Role |
|---|---|
| `src/main.rs` | Binary entry-point; installs rustls crypto provider, creates `App`, calls `.run().await` |
| `src/app.rs` | `App` struct — wiring: token resolve → DiscordClient → gateway + command loop → `tui::run` |
| `src/tui/runtime.rs` | Main TUI event loop: `tokio::select!` over terminal events, Discord snapshots, effects, redraw/image-decode timers |
| `src/tui/state.rs` | `DashboardState` — the everything state machine (panes, filter, composer, popups, ack, …); sub-modules in `tui/state/` |
| `src/tui/media/` | Image preview / avatar / emoji LRU caches, protocol glue for `ratatui-image` |
| `src/tui/input.rs` | Keyboard + mouse dispatchers (`handle_key`, `handle_paste`, `handle_mouse_event`) |
| `src/tui/terminal.rs` | `TerminalRestoreGuard` — manages crossterm terminal modes across app lifetime |
| `src/tui/login.rs` | Auth screens (token, email/password, QR, MFA) |
| `src/tui/ui/mod.rs` | ratatui `render()` + pane layout helpers |
| `src/tui/commands.rs` | State → `AppCommand` bridges |
| `src/tui/effects.rs` | Effect-processing that mutates state from Discord events |
| `src/tui/redraw.rs` | Dirty-flag / signature-diff to avoid unnecessary OSC 1337 redraws |
| `src/discord/client.rs` | `DiscordClient` — gateway + REST orchestration |
| `src/discord/rest.rs` | HTTP helpers |
| `src/discord/gateway.rs` | WebSocket gateway event loop |
| `src/discord/events.rs` | `AppEvent`, `SequencedAppEvent`, all event types |
| `src/discord/commands.rs` | `AppCommand` enum and argument types |
| `src/discord/state.rs` | `DiscordState`, `DiscordSnapshot`, read-model |
| `src/config.rs` | `DisplayOptions` — persists/loads `config.toml` |
| `src/token_store.rs` | Load / save Discord token (plaintext credential file) |
| `src/logging.rs` | File + in-memory error logger |
| `src/paths.rs` | All file paths (config, credential, log, downloads) |
| `src/error.rs` | `AppError`, `Result<T>` — shared error types |
| `src/version_check.rs` | Polls crates.io sparse index to detect newer concord releases |

**There is no workspace, no Makefile, no Justfile.** Cargo is the only build tool.

---

## 2 · Exact developer commands

Run before every push (CI runs the same set — see `CONTRIBUTING.md → §Before you push`):

```pwsh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Run locally during development:

```pwsh
cargo run               # build + launch
cargo build --release   # release binary at target/release/concord
cargo build             # debug build
cargo test              # unit tests
cargo clippy            # lint
cargo fmt               # format in-place
```

Bun wrapper (runs the built binary):
```pwsh
bun install             # installs dependencies
bun run concord         # runs the concord binary (builds if needed)
```

CI additionally runs `cargo dist plan` to catch broken release config.

### CI check order
```
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo dist plan        # validate dist-workspace.toml
```

### Run a single test / filter

```pwsh
cargo test --all-features -- function_name          # by test name
cargo test --all-features -- path::to::module        # by module path
cargo test --all-features -- --list                  # list every test
```

---

## 3 · Toolchain requirements

| Item | Value |
|---|---|
| Rust edition | 2024 |
| MSRV | 1.85 (`Cargo.toml → rust-version = "1.85"`) |
| Package manager | Cargo (primary), Bun (for npm wrapper) |

---

## 4 · Architecture — key structural facts

### Command / effect loop

```
App::run
  → gateway_task (DiscordClient::start_gateway)
  → command_task (start_command_loop — REST calls, publishes AppEvent effects)
  → tui::run (ratatui event loop in runtime.rs)
       tokio::select! {
           terminal events → events::handle_terminal_event → state / AppCommand
           image-decode rx  → ImagePreviewCache.store_decoded
           snapshot change  → state.restore_discord_snapshot + deferred effects
           sequenced effect → effects::process_sequenced_effect
           timers           → redraw debounce (80 ms) / read-ack flush / toast expiry
       }
       auto-sends: history, pinned-messages, forum posts, members,
                   profiles, thread previews, member-list subscriptions
```

### Two top-level entries into `media.rs`

- `AvatarImageCache` — LRU for user avatar images
- `EmojiImageCache` — LRU for custom emoji images
- `ImagePreviewCache` — 16-entry cap, Loading → Decoding → Ready/Failed state machine

### Command sender boundary

`tui/runtime.rs` sends `AppCommand` values north to `app.rs::start_command_loop`
via a `tokio::sync::mpsc::Sender<AppCommand>`.  `start_command_loop` spawns each
command as an independent task (slow REST calls do not block the UI queue).

---

## 5 · Testing quirks

- All tests are inline `#[cfg(test)] mod tests` in the same file — **no separate `tests/` directory**.
- Tests use `ratatui::backend::TestBackend` for render assertions.
- Temporary directories in `std::env::temp_dir()` — clean up in the test.
- **Do NOT add `#[should_panic]` assertions.** Never `unwrap()` a `Result` unless
  the enclosing `#[test]` already returns `Result<()>`.
- Image-related tests decode actual image bytes and can be slow — use `-- <filter>`.
- Run `cargo test --all-features` before pushing; that is what CI runs.

---

## 6 · Build / config / runtime quirks

### Image prototype detection
`src/tui/media/protocol.rs:query_image_picker` calls
`ratatui_image::picker::Picker::from_query_stdio()` once at startup.
Graceful fallback order: **Kitty Graphics → iTerm2 → Sixel → Halfblocks**.
Silently degrades so the app stays usable on any terminal.

### File paths (src/paths.rs)
| env | path |
|---|---|
| `XDG_CONFIG_HOME/concord/config.toml` | config (ignores relative `XDG_CONFIG_HOME`) |
| `XDG_CONFIG_HOME/concord/credential` | Discord token plaintext; `0600` on unix |
| `XDG_CONFIG_HOME/concord/concord.log` | log file |
| `dirs::download_dir() \| ~/Downloads` | attachment downloads |

### Logging
| env var | effect |
|---|---|
| `CONCORD_DEBUG=1` | `DEBUG` + `TIMING` lines written to log file |
| `CONCORD_LOG_FILE=/path` | override default log path |
| `` ` `` (backtick) key | opens in-app debug popup (last 200 error entries) |

Error logging: `logging::error("target", format!(…))`.  All targets are
module names: `"history"`, `"preview"`, `"app"`, `"tui"`, `"config"`.

### Version check (src/version_check.rs)
Polls `https://index.crates.io/co/nc/concord` — the crates.io sparse index.
`ComparableVersion` is hand-rolled (no `semver` dep).  Ignores yanked and
pre-release unless the current version itself is pre-release.

### Open URL
Windows: `cmd /C start "" <url>`; macOS: `open <url>`; Linux: `xdg-open <url>`.

### Token storage
Plaintext, no keychain.  Unix: `0600` file, `0700` parent dir.
If credential store fails, the user is prompted to re-enter the token each session.

### ATTACHMENT upload limits (app.rs constants)
| constant | value |
|---|---|
| `MAX_ATTACHMENT_PREVIEW_BYTES` | 8 MiB per HTTP fetch |
| `MAX_ATTACHMENT_DOWNLOAD_BYTES` | 64 MiB per download |
| `MAX_CONCURRENT_ATTACHMENT_PREVIEWS` | 4 (tokio Semaphore) |
| `MESSAGE_HISTORY_LIMIT` | 50 |

Redraw debounce (runtime.rs): `BACKGROUND_REDRAW_DEBOUNCE = 80 ms`.

---

## 7 · Windows-specific

| Quirk | Detail |
|---|---|
| Build / run | Use PowerShell commands above |
| Open URL | `cmd /C start "" url` (note the `""` which is the window title) |
| External editor | `EDITOR` env var; runs via `sh -c "$EDITOR \"$1\" -- <file>"` |
| `chmod 0600/0700` | No-ops on non-Unix (stock `write_private_file` path) |
| `CONCORD_DEBUG` | Works from any shell (env var) |

---

## 8 · Environment variables for release
| Variable | Purpose |
|---|---|
| `CARGO_REGISTRY_TOKEN` | `cargo publish --locked` |
| `GITHUB_TOKEN` | `dist` GitHub operations, release creation |
| `GITHUB_TOKEN` (HOMEBREW_TAP_TOKEN) | Formula push to `chojs23/homebrew-tap` |

---

## 9 · Conventions and gotchas

- **One command → one `AppEvent` publish.** All REST failures are logged and
  surface as `AppEvent::GatewayError` via the effect stream.
- **Deferred effects buffer.** `SequencedAppEvent.revision` must be ≥ the
  current snapshot revision before the effect is applied.  Stale effects are
  held in `deferred_effects` and drained when the snapshot catches up.
- **Member list subscription (runtime.rs).** On guild open the code sends
  `SubscribeDirectMessage` + `SubscribeGuildChannel` for **every** viewable
  channel.  On scroll into a new 100-member bucket it sends
  `UpdateMemberListSubscription` with the new range to keep member rows fresh.
- **Unread anchor scroll.** When opening a channel with unread messages the
  viewport initially auto-follows the latest message; `try_apply_unread_anchor_scroll`
  re-anchors to the "last read" position once the ack snapshot arrives.
- **Guild/member list uses `leader-action (`Cha < 80ms for Channels pane, members (`.c.then` or `member_view_height` in `src/tui/runtime.rs`).
- **Thread view spilling** — not in `tui/runtime.rs`, but in `tui/state.rs`; `is_thread_view_full(cfg)` is the right approach; see `src/tui/state/discord/state.rs`.
- Image preview quality rewriting (`ImagePreviewQualityPreset`) applies to
  attachment/embed previews only, **not** avatars or custom emoji.
- Pinned message pins preserved across history reload — `src/tui/state/pinned.rs`.

---

## 10 · Tests only: `#[test]` → `Result<()>`

Some `#[test]` functions return `Result<()>` and use `?` for cleanup errors.
Those are fine.  Never introduce `#[should_panic]` or `unwrap()`.

---

_Last updated: analysis of the concord repo at `D:\repos\concord`_
