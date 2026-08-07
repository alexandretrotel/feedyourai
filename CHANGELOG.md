# Changelog

All notable changes to this project will be documented in this file.

## Unreleased - 3.0.0

Added
- `fyai` as a second binary target, aliasing `feedyourai` (same CLI, no behavior difference).
- `error` module with a `FyaiError` type (via `thiserror`), replacing `color-eyre` in the library crate.

Changed
- **Breaking:** CLI wiring (`cli`, `init`, clipboard) moved out of the library into the `fyai`/`feedyourai` binaries; the library no longer prints to stdout/stderr or copies to the clipboard, and no longer depends on `color-eyre`.
- Replaced `serde_yaml` (archived) with `yaml_serde`.
- Replaced `directories-next` with `dirs` for locating system config directories.
- `documentation` field in `Cargo.toml` now points to `docs.rs` instead of the GitHub repo.
- README demo GIF now uses a raw GitHub link so it renders on crates.io.
- CI split into `ci.yml` (fmt, clippy, machete, test), `build-binaries.yml`, and `release.yml`, each triggered on `push` to `main` in addition to pull requests.
- `release.yml` now uploads both `fyai` and `feedyourai` binaries per target.

Added (crate metadata)
- `keywords`, `categories`, and `readme` fields for crates.io discoverability.

## 2.1.3 - 2026-07-31

Changed
- The published crate now uses an explicit `include` allowlist instead of an `exclude` denylist, so only `src/`, `Cargo.toml`, `README.md`, `LICENSE`, and `CHANGELOG.md` are shipped. `.gitignore` and any future non-source files no longer end up in the package.

## 2.1.0 - 2026-07-18

Changed
- Switched error handling across the crate to `eyre` (with `color-eyre` reporting in the `fyai` binary), using native `eyre` macros (`eyre!`, `bail!`, `wrap_err`, `ok_or_eyre`).
- `run_local` and `run_git` now return `eyre::Result<()>`.

Removed
- Removed the `thiserror` dependency and the `errors` module (`AppError` / `AppResult`).

## 2.0.3 - 2026-05-13

Changed
- Removed unused `anyhow` dependency.

## 2.0.2 - 2026-02-18

Changed
- Split CLI-specific logic into a `cli` module folder.
- Exposed a library API with `run_local` and `run_git`, keeping lower-level modules public.
- Removed `clap` usage from the library crate.

## 2.0.1 - 2026-02-17

Fixed
- Added a tree structure header to the output.
- Use `--input` instead of `--directory` in CLI handling.

Changed
- README updates and corrections.

## 2.0.0 - 2026-02-18

Changed
- Switched traversal to `ignore::WalkBuilder` with standard ignore filters, which now honors `.gitignore`, `.git/info/exclude`, global gitignore, `.ignore`, and hidden files by default.
- Replaced `dirs` with `directories-next` for locating system config directories.

## 1.7.2 - 2026-02-09

Changed
- Replaced `clipboard` with `arboard` to avoid the `xcb` dependency and CMake build step.

## 1.7.1 - 2026-02-09

Changed
- Updated dependencies: thiserror 1.0 -> 2.0.18.

## 1.7.0 - 2026-02-09

Added
- `--repo` to process a remote git repository in a temporary directory.
- `--repo-branch` to checkout a branch or tag when using `--repo`.
- `--repo-commit` to checkout a specific commit when using `--repo`.
- Repository integration tests covering clone, cleanup, and commit checkout.

Changed
- Error handling now uses typed `AppError` variants (via `thiserror`), removing string-based checks for clipboard and config errors.
- When `--repo-commit` is used, cloning no longer uses `--depth 1` to ensure the commit is available.
- Documentation updated with a remote-repo usage example.
- Applied a formatting pass to keep code style consistent.

Fixed
- CLI now prevents using `--repo` and `--dir` together.
