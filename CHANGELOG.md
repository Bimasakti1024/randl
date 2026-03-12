# Changelog

All notable changes to this project will be documented in this file.

## [0.3.1] - 2026-03-12
### Security
- updated `quinn-proto` to 0.11.14 to patch a potential Denial of Service vulnerability

## [0.3.0] - 2026-03-07
### Added
- `repo` alias for `repository` subcommand
- `--no-confirm` flag to skip confirmation before downloading
- `--repeat` flag to repeat pulling
- `--timeout` flag for downloading timeout at pulling reward or nested repository following
- `check` to find dead repository

### Changes
- repository synchronization can now be targetting a repository instead of synchronizing all repositories.

## [0.2.0] - 2026-03-04
### Added
- `--output-directory` flag to save downloads to a specific directory
- `--dry-run` flag to preview reward without downloading
- Sync now shows how many repositories succeeded and failed

### Fixed
- Nested repository results are now properly propagated
- Empty lines and comments are now filtered in nested repository too

## [0.1.0] - 2026-03-03
### Initial Release
- Add, remove, list, and sync repositories
- Pull a random file from synced repos
- Nested repository support
- File size preview and confirmation before downloading
