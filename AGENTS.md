# Repository Guidelines

## Project Structure & Module Organization
- Entry point at `src/main.rs` wires the clap CLI to subcommands; shared exports live in `src/lib.rs`.
- CLI subcommands in `src/cli/` (`clone`, `mass`, `quick`, `profile`, `alias`, `update`) orchestrate workflows and regex filtering.
- Provider clients in `src/git_api/` (GitHub/GitLab) and profile/config handling in `src/config/`; helper utilities (command runners, regex args) sit in `src/utils/`.
- Packaging is defined by `Cargo.toml`; build outputs land in `target/`; transient logs (if any) stay in `build/logs/`.

## Build, Test, and Development Commands
- `cargo check` for quick validation; `cargo build --release` to produce the optimized binary.
- `cargo run -- --help` to view usage; example: `cargo run -- clone org-name --regex ".*" --dry-run`.
- `cargo fmt --all` enforces formatting; `cargo clippy --all-targets --all-features` catches lints (fix or allow with rationale).
- `cargo test` runs the suite; use `cargo test -- --nocapture` when debugging print output.

## Coding Style & Naming Conventions
- Rust 2021 edition; keep `rustfmt.toml` defaults; run `cargo fmt` before commits.
- Prefer clear error contexts; bubble errors with `Result` and avoid `expect`/`unwrap` outside tests or truly fatal conditions.
- Modules and functions use `snake_case`; types `PascalCase`; constants `SCREAMING_SNAKE_CASE`.
- Keep CLI prompts and user-visible strings concise; prefer `&str` over `String` when borrowing is enough.

## Testing Guidelines
- Add unit tests alongside modules with `#[cfg(test)]`; integration tests may live in `tests/` if cross-module.
- For CLI flows, prefer testing command-building helpers instead of shelling out; stub network calls when possible.
- Aim to cover new branches for provider handling (GitHub/GitLab) and regex filtering paths before merging.

## Commit & Pull Request Guidelines
- Follow conventional commit prefixes seen in history (`chore:`, `fix:`, `refactor:`, `style:`); keep messages imperative and under ~72 chars.
- PRs should describe intent, note affected commands/flags, and list manual test steps (commands run with results).
- Link related issues or TODOs; include output snippets or screenshots for user-facing changes.

## Configuration & Security
- User profiles live in `~/.config/grgry.toml`; never commit tokens or personal paths.
- Use `--dry-run` flags when validating mass operations; avoid logging secrets returned from providers.
