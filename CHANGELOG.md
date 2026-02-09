# Changelog

All notable changes to this project will be documented in this file.

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
