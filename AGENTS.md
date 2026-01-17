# AGENTS.md

## Project overview
- GPUI-based clipboard manager for Windows.
- Clipboard polling uses `clipboard-win`.
- Clipboard history is stored in a local SQLite database via `sea-orm`.

## Responsibilities and layout
- UI: `src/ui/`
- Clipboard polling: `src/clipboard/`
- Storage (DB path, schema, queries): `src/storage/`
- App wiring and lifecycle: `src/app.rs`

## Conventions
- Keep modules small and focused; avoid mixing UI, clipboard access, and DB logic.
- Prefer `anyhow::Result` for fallible operations at module boundaries.
- Windows-only clipboard code should be gated with `#[cfg(target_os = "windows")]`.

## Database
- DB file is `clipboard_history.db` under `%LOCALAPPDATA%\gpui-clipboard-manager`.
- Image files live in `%LOCALAPPDATA%\gpui-clipboard-manager\clipboard_images`.

- Schema lives in `src/migration/` and should be the single source of truth.

## Dev notes
- Default run: `cargo run`
- No automated tests yet.
- To generate a new migration: `sea-orm-cli migrate generate <name>`
- To apply migrations manually: `sea-orm-cli migrate up`
