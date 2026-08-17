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

The application expects `JAVA_HOME`, `ANDROID_HOME`/`ANDROID_SDK_ROOT`, and `ANDROID_NDK_HOME` to be configured by the computer environment. The repository does not modify the Java toolchain.

### Build and install on a connected Android phone

```bash
scripts/install.sh
# Or choose a device explicitly:
scripts/install.sh -s DEVICE_SERIAL
```

The installer checks the JDK/SDK/NDK, selects the sole connected `arm64-v8a` device when possible, makes a clean Dioxus Android build, verifies that the APK contains `lib/arm64-v8a/libmain.so`, installs with `adb install -r`, and verifies the package afterward. `-r` preserves the app-private SQLite database.

Selection order is `-s`, `ANDROID_SERIAL`, then the sole compatible device. If several compatible devices are connected, the script stops and asks for `-s`; if only a 32-bit `armeabi-v7a` phone is attached, it stops before building because Dioxus 0.7 cannot target it.

The debug APK is emitted at:

```text
target/dx/china-travel-app/debug/android/app/app/build/outputs/apk/debug/app-debug.apk
```

## App icon

`assets/icon.png` is the canonical star-free route-and-location travel icon. The same PNG is rendered at the top-left of the app header and rasterized into the committed Android launcher density images under `assets/android-launcher/`. `scripts/install.sh` applies those PNGs after Dioxus stages its stock Android launcher resources; storing them outside `mobile/android/res/` and omitting a launcher icon from `Dioxus.toml` avoids Dioxus 0.7.10's duplicate Android PNG/WebP resource collision.

To regenerate all icon sizes after changing the design generator:

```bash
python3 scripts/generate_icon.py
```
