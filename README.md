# Record

Record is a compact local task panel. The repo contains two runnable builds:

- `src-tauri`: Tauri + React version with an NSIS installer.
- `native-record`: native Rust/egui version for a single executable without WebView startup cost.

## Tauri Commands

- `npm run dev` starts the Vite frontend.
- `npm run tauri dev` starts the desktop app in development mode.
- `npm run test` runs the React and task-helper tests.
- `npm run build` builds the frontend.
- `npm run tauri build` builds the app and a single-file NSIS installer.

Task data is stored locally in the app data directory as `tasks.json`.

The Windows NSIS installer explicitly bundles `WebView2Loader.dll` next to `record.exe`, so the installed app can start without requiring that loader to be present on `PATH`.

The main window starts hidden and is shown only after the first local task load finishes, which avoids displaying an unfinished WebView during startup.

## Native Single-File Commands

- `cargo test --manifest-path native-record/Cargo.toml` runs the native task-store tests.
- `cargo build --release --manifest-path native-record/Cargo.toml` builds the native executable.

The native Windows executable is generated at `native-record/target/release/record-native.exe`.

The native build does not use WebView2 or a browser shell. The app icon is embedded from `src-tauri/icons/icon.ico`, and the runtime window icon is embedded from `src-tauri/icons/icon.png`.
