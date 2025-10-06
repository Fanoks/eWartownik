eWartownik
================================

A small desktop GUI application built with Rust and Slint. This is a temporary README to help you build, run, and understand the basics of the project layout.

Features
- Rust + Slint UI (native-looking, cross‑platform)
- Bundled translations (i18n) using Slint; current locales in `lang/`
- SQLite ready via `rusqlite` (for future data persistence)

Requirements
- Rust (stable) and Cargo
- Windows: MSVC toolchain (install “Desktop development with C++” or the standalone Build Tools)

Build and run
Use PowerShell from the project root:

```powershell
# Run in debug mode
cargo run

# Build a release binary
cargo build --release
# The binary will be at: .\target\release\eWartownik.exe
```

Internationalization (i18n)
- Translation files live under `lang/<locale>/LC_MESSAGES/eWartownik.po`.
- Translations are bundled at build time via `build.rs` (see `with_bundled_translations("lang")`).
- The app currently selects Polish by default in `src/main.rs` using:
	`slint::select_bundled_translation("pl");`
- To change the default language, adjust the code above or call it with a different locale code (e.g., `"en"`, `"de"`, `"jp"`).

Project structure
- `src/main.rs` – Rust entry point and app setup
- `ui/` – Slint UI files (`app.slint`, `title.slint`, icons in `images/`)
- `build.rs` – Slint build configuration (bundled translations, style)
- `lang/` – Translation catalogs (PO files per locale)
- `Cargo.toml` – Dependencies and package metadata

Notes
- The translation template `messages.pot` is currently ignored by Git (see `.gitignore`). If you want it tracked, remove the line or force-add it.

