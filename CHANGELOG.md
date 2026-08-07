# Changelog

All notable changes to this project will be documented in this file.

## Unreleased - 3.0.0

Added
- `fyai` as a second binary target, aliasing `feedyourai` (same CLI, no behavior difference).
- `error` module with a `FyaiError` type (via `thiserror`), replacing `color-eyre` in the library crate.
- `--human` flag (and `human` config-file key) to render the directory tree with `tree`-style connector glyphs (`├──`, `└──`, `│`) instead of the default minimal two-space indent.
- `system_config_dir` now honors `$XDG_CONFIG_HOME` (when set to an absolute path) on every platform, not just Linux, before falling back to the platform default.

Changed
- **Breaking:** CLI wiring (`cli`, `init`, clipboard) moved out of the library into the `fyai`/`feedyourai` binaries; the library no longer prints to stdout/stderr or copies to the clipboard, and no longer depends on `color-eyre`.
- Replaced `serde_yaml` (archived) with `yaml_serde`.
- Replaced `directories-next` with `dirs` for locating system config directories.
- `documentation` field in `Cargo.toml` now points to `docs.rs` instead of the GitHub repo.
- README demo GIF now uses a raw GitHub link so it renders on crates.io.
- CI split into `ci.yml` (fmt, clippy, machete, test), `build-binaries.yml`, and `release.yml`, each triggered on `push` to `main` in addition to pull requests.
- `release.yml` now uploads both `fyai` and `feedyourai` binaries per target.
- `IGNORED_DIRS` trimmed to committed config/VCS directories only (`.github`, `.vscode`, `.git`, etc., grouped and doc-commented with a note to re-sync against github/gitignore's `Global/` templates); build/dependency/cache directories (`node_modules`, `target`, `.venv`, ...) are no longer hardcoded and now rely on `.gitignore` via `respect_gitignore`.
- `get_directory_structure` now dispatches between two renderers based on `config.human`: a minimal two-space indent by default, or `tree`-style ASCII connectors when set.
- **Breaking:** Per-file output format changed from `- File: name (size bytes)` + raw content to a `### relative/path (human size)` heading followed by a language-tagged, fenced code block (language inferred from extension via the new `scanner::lang` module). The fence widens from ```` ``` ```` to ```` ```` ```` when the file's own content contains a triple backtick, so the block's end is never ambiguous — the old format had no closing delimiter at all.
- **Breaking:** `merge_config` now takes two `FileConfig`s (`file`, `cli`) instead of a `FileConfig` plus a fully-resolved `Config` and a separate `ExplicitFlags`. CLI parsing (`config_from_matches`) now returns a `FileConfig` directly, leaving a field `None` unless it was explicitly passed, instead of always resolving to a concrete value and tracking "was this explicit" on the side.
- Wording: replaced "AI" with "LLM" throughout descriptions and doc comments (binary names `feedyourai`/`fyai` unchanged).

Fixed
- `--repo`'s `conflicts_with` referenced a nonexistent `directory` arg id (the actual id is `input`), which made every CLI invocation panic at startup.
- `--no-gitignore` detection looked up the wrong arg id (`respect_gitignore`, which doesn't exist) and tried to parse it as a string; the flag was never actually read. Now correctly negates the `no_gitignore` `SetTrue` flag.
- CLI `--help` and `init --global`'s help text hardcoded `~/.config/fyai.yaml` as the global config path, which is wrong on macOS/Windows and ignores `$XDG_CONFIG_HOME`. Now describes the actual resolution and points to `fyai init --global` for the exact path.

Removed
- `IGNORED_FILES` constant and the hardcoded lockfile skip list. Use `exclude_files` to skip specific file names.
- `ExplicitFlags` struct — superseded by `FileConfig`-based CLI parsing (see Changed).

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
