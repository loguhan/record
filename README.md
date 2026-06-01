# Record

Record is a compact local task panel. The recommended distributable is the native build in `native-record`, which does not use WebView, Node, Tauri, or a browser shell at runtime.

## User Downloads

There is no single binary that can run on every operating system, because Windows, Linux, and macOS use different executable formats. The no-environment release model is:

- Windows users download `Record-windows-x64.zip` and run `Record.exe`.
- Linux users download `Record-linux-x64.tar.gz`, extract it, and run `Record`.
- macOS Intel users download `Record-macos-x64.tar.gz`, extract it, and run `Record.app`.
- macOS Apple Silicon users download `Record-macos-arm64.tar.gz`, extract it, and run `Record.app`.

Users do not need Rust, Node.js, npm, Tauri, or a development environment.

## Release Build

The GitHub Actions workflow `.github/workflows/native-release.yml` builds all supported desktop packages automatically on the correct operating systems. Run the `native-release` workflow manually, or push a tag such as `v0.1.0`.

Artifacts produced by the workflow:

- `Record-windows-x64.zip`
- `Record-linux-x64.tar.gz`
- `Record-macos-x64.tar.gz`
- `Record-macos-arm64.tar.gz`

## Local Developer Commands

Native app:

```powershell
cargo test --manifest-path native-record/Cargo.toml
cargo build --release --manifest-path native-record/Cargo.toml
```

Windows native executable:

```text
native-record/target/release/record-native.exe
```

Tauri/React is kept in `src-tauri` and `src` as an optional webview-based build, but the native build is the preferred no-environment distribution path.

Task data is stored locally in the OS app-data directory as `Record/tasks.json`.
