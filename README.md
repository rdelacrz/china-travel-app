# China Travel Companion

Android-first local travel management app for China, built with Rust, Dioxus 0.7, Tailwind CSS v4, SQLite, and the official Dioxus Components generated from the pinned DioxusLabs repository revision.

## Features

- Home overview with persisted trips, checklist/document counts, progress, and zero-state summaries.
- Checklist items with immediate checkbox persistence, inline edit-on-tap, blur/keyboard-Go commits, validation, and AlertDialog delete confirmation.
- Documentation notes with right-side Sheet add/edit form, multiline descriptions, linkified HTTP(S) URLs, view/edit/open-file actions, and optional Android Storage Access Framework attachments.
- Android bridge for app-data directory lookup, `ACTION_OPEN_DOCUMENT`, persisted read grants, `ACTION_VIEW` for attachments, browser URL opening, and post-commit grant release.
- Structured SQLite migrations and repositories using `tokio-rusqlite` with bundled SQLite.

## Repository layout

```text
android/MainActivity.kt       Android WebView bridge and SAF activity result handling
migrations/0001_initial.sql  SQLite schema
src/app.rs                    Routes, startup, service context, shell integration
src/components/               Generated Dioxus Components plus behaviorful panes
src/db/                       SQLite connection, migration, and repositories
src/domain/                   Validated trip/checklist/document models
src/platform/                 Android protocol/implementation and host fake
src/views/                    home.rs, checklist.rs, documentation.rs
 tests/                       Database, bridge protocol, and platform workflow tests
```

## Development checks

Host tests and strict linting do not require Android rendering:

```bash
cargo fmt --all
cargo test --no-default-features
cargo clippy --no-default-features --all-targets -- -D warnings
```

The application expects `JAVA_HOME`, `ANDROID_HOME`/`ANDROID_SDK_ROOT`, and `ANDROID_NDK_HOME` to be configured by the computer environment. The repository does not modify the Java toolchain. Dioxus 0.7 requires a 64-bit Android target; the supported package target here is `aarch64-linux-android`:

```bash
# Use the JDK/SDK/NDK paths provided by the environment.
dx build --android --target aarch64-linux-android
```

If the NDK exposes only API-versioned clang wrappers, set the target C compiler and linker for the build shell, for example:

```bash
export NDK_BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android35-clang"
export CXX_aarch64_linux_android="$NDK_BIN/aarch64-linux-android35-clang++"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_BIN/aarch64-linux-android35-clang"
dx build --android --target aarch64-linux-android
```

The debug APK is emitted at:

```text
target/dx/china-travel-app/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
```

Install on a 64-bit Android device with:

```bash
adb install -r target/dx/china-travel-app/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
```

The connected validation phone currently advertises only `armeabi-v7a`; Dioxus 0.7 rejects 32-bit targets and the resulting arm64 APK cannot install on that phone (`INSTALL_FAILED_NO_MATCHING_ABIS`). Use a 64-bit Android device or emulator for runtime validation.
