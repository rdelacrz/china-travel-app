#!/usr/bin/env bash
set -Eeuo pipefail

# Build the Android debug APK for a connected ARM64 device and install it without
# clearing the app's private SQLite database.
#
# Usage:
#   scripts/install.sh
#   scripts/install.sh -s DEVICE_SERIAL
#
# Selection order: -s, ANDROID_SERIAL, then the sole connected ARM64 device.
# Dioxus 0.7 does not support the 32-bit Android targets used by armeabi-v7a-only
# phones, so this script intentionally requires arm64-v8a.

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="$ROOT_DIR/target"
SERIAL="${ANDROID_SERIAL:-}"

usage() {
    cat <<'EOF'
Usage: scripts/install.sh [-s DEVICE_SERIAL]

Build the China Travel Companion Android debug APK and install it on a connected
ARM64 Android device. The existing app-private SQLite database is preserved.

Selection order:
  1. -s DEVICE_SERIAL
  2. ANDROID_SERIAL environment variable
  3. The sole connected arm64-v8a device

Options:
  -s SERIAL  Target this Android device serial.
  -h         Show this help message.

Dioxus 0.7 requires a 64-bit Android device. An armeabi-v7a-only phone cannot
install this application's arm64-v8a APK.
EOF
}

die() {
    printf 'ERROR: %s\n' "$*" >&2
    exit 1
}

on_error() {
    local status=$?
    printf 'ERROR: command failed with exit %d: %s\n' "$status" "$BASH_COMMAND" >&2
}

trap on_error ERR

while getopts ':s:h' option; do
    case "$option" in
        s)
            [[ -n "$OPTARG" ]] || die 'The -s option requires a device serial.'
            SERIAL="$OPTARG"
            ;;
        h)
            usage
            exit 0
            ;;
        :)
            die "Option -$OPTARG requires an argument."
            ;;
        \?)
            die "Unknown option: -$OPTARG. Use -h for help."
            ;;
    esac
done
shift $((OPTIND - 1))
[[ "$#" -eq 0 ]] || die "Unexpected argument: $1. Use -h for help."

[[ -f "$ROOT_DIR/Cargo.toml" ]] || die "Cargo.toml not found from project root: $ROOT_DIR"
[[ -f "$ROOT_DIR/Dioxus.toml" ]] || die "Dioxus.toml not found from project root: $ROOT_DIR"

if [[ -z "${JAVA_HOME:-}" || ! -x "$JAVA_HOME/bin/java" ]]; then
    die 'JAVA_HOME must point to a JDK with bin/java. Configure it before running this script.'
fi

ANDROID_HOME="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
export ANDROID_HOME
export ANDROID_SDK_ROOT="$ANDROID_HOME"
NDK_DIR="${ANDROID_NDK_HOME:-${NDK_HOME:-$ANDROID_HOME/ndk/30.0.14904198}}"
export NDK_HOME="$NDK_DIR"
export ANDROID_NDK_HOME="$NDK_DIR"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"

[[ -d "$ANDROID_HOME" ]] || die "Android SDK not found: $ANDROID_HOME"
[[ -d "$ANDROID_NDK_HOME" ]] || die "Android NDK not found: $ANDROID_NDK_HOME"

ADB="${ADB:-$ANDROID_HOME/platform-tools/adb}"
[[ -x "$ADB" ]] || die "adb not found or not executable: $ADB"

DX="$(command -v dx || true)"
[[ -n "$DX" ]] || die 'The Dioxus CLI (dx) was not found on PATH.'

RUSTUP="$(command -v rustup || true)"
[[ -n "$RUSTUP" ]] || die 'rustup was not found on PATH.'

UNZIP="$(command -v unzip || true)"
[[ -n "$UNZIP" ]] || die 'unzip is required to verify the APK ABI.'

PACKAGE_NAME="$(awk -F'"' '/^[[:space:]]*name[[:space:]]*=/ { print $2; exit }' "$ROOT_DIR/Cargo.toml")"
APPLICATION_ID="$(awk -F'"' '/^[[:space:]]*identifier[[:space:]]*=/ { print $2; exit }' "$ROOT_DIR/Dioxus.toml")"
MIN_SDK="$(awk -F'=' '/^[[:space:]]*min_sdk[[:space:]]*=/ { gsub(/[[:space:]]/, "", $2); print $2; exit }' "$ROOT_DIR/Dioxus.toml")"
COMPILE_SDK="$(awk -F'=' '/^[[:space:]]*compile_sdk[[:space:]]*=/ { gsub(/[[:space:]]/, "", $2); print $2; exit }' "$ROOT_DIR/Dioxus.toml")"
[[ -n "$PACKAGE_NAME" ]] || die 'Could not read the package name from Cargo.toml.'
[[ -n "$APPLICATION_ID" ]] || die 'Could not read the Android application ID from Dioxus.toml.'
[[ "$MIN_SDK" =~ ^[0-9]+$ ]] || die 'Could not read a numeric min_sdk from Dioxus.toml.'
[[ "$COMPILE_SDK" =~ ^[0-9]+$ ]] || die 'Could not read a numeric compile_sdk from Dioxus.toml.'

ICON_DENSITIES=(mdpi hdpi xhdpi xxhdpi xxxhdpi)
ICON_RES_DIR="$ROOT_DIR/assets/android-launcher"
for density in "${ICON_DENSITIES[@]}"; do
    [[ -f "$ICON_RES_DIR/mipmap-$density/ic_launcher.png" ]] || \
        die "Android launcher icon is missing for $density: $ICON_RES_DIR/mipmap-$density/ic_launcher.png"
done

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)
        NDK_HOST_TAG='linux-x86_64'
        ;;
    Linux-aarch64)
        NDK_HOST_TAG='linux-aarch64'
        ;;
    *)
        die "Unsupported host for this script: $(uname -s)-$(uname -m)"
        ;;
esac

NDK_BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$NDK_HOST_TAG/bin"
[[ -d "$NDK_BIN" ]] || die "Android NDK toolchain directory not found: $NDK_BIN"
CLANG="$NDK_BIN/aarch64-linux-android${COMPILE_SDK}-clang"
[[ -x "$CLANG" ]] || die "Android NDK compiler not found: $CLANG"
[[ -x "$NDK_BIN/llvm-ar" ]] || die "Android NDK archiver not found: $NDK_BIN/llvm-ar"

# Keep direct Cargo/cc-rs dependencies aligned with the Dioxus Android target.
export CC_aarch64_linux_android="$CLANG"
export CXX_aarch64_linux_android="${CLANG}++"
export AR_aarch64_linux_android="$NDK_BIN/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CLANG"

"$ADB" start-server >/dev/null
DEVICE_TABLE="$("$ADB" devices)"

device_state() {
    awk -v serial="$1" '$1 == serial { print $2; exit }' <<<"$DEVICE_TABLE"
}

device_abis() {
    local serial=$1
    local abis
    abis="$("$ADB" -s "$serial" shell getprop ro.product.cpu.abilist </dev/null | tr -d '\r')"
    if [[ -z "$abis" ]]; then
        abis="$("$ADB" -s "$serial" shell getprop ro.product.cpu.abi </dev/null | tr -d '\r')"
    fi
    printf '%s' "$abis"
}

select_device() {
    local candidate
    local candidate_abis
    local -a compatible=()
    local -a connected=()

    if [[ -n "$SERIAL" ]]; then
        return
    fi

    mapfile -t connected < <(printf '%s\n' "$DEVICE_TABLE" | awk 'NR > 1 && $2 == "device" { print $1 }')
    for candidate in "${connected[@]}"; do
        [[ -n "$candidate" ]] || continue
        candidate_abis="$(device_abis "$candidate")"
        if [[ "$candidate_abis" == *arm64-v8a* ]]; then
            compatible+=("$candidate")
        fi
    done

    case "${#compatible[@]}" in
        1)
            SERIAL="${compatible[0]}"
            ;;
        0)
            die "No connected arm64-v8a Android device was found. Connected devices:"$'\n'"$DEVICE_TABLE"
            ;;
        *)
            die "Multiple arm64-v8a devices are connected. Select one with -s DEVICE_SERIAL."$'\n'"$DEVICE_TABLE"
            ;;
    esac
}

select_device
DEVICE_STATE="$(device_state "$SERIAL")"
case "$DEVICE_STATE" in
    device)
        ;;
    unauthorized)
        die "Device $SERIAL is unauthorized. Unlock it and accept the USB debugging prompt."
        ;;
    offline)
        die "Device $SERIAL is offline. Reconnect it and retry."
        ;;
    '')
        die "Target Android device $SERIAL was not found. Connected devices:"$'\n'"$DEVICE_TABLE"
        ;;
    *)
        die "Device $SERIAL has unexpected adb state: $DEVICE_STATE"
        ;;
esac

ABI_LIST="$(device_abis "$SERIAL")"
[[ "$ABI_LIST" == *arm64-v8a* ]] || die "Device $SERIAL reports unsupported ABI(s): ${ABI_LIST:-<empty>}. Dioxus 0.7 requires arm64-v8a."

DEVICE_API="$("$ADB" -s "$SERIAL" shell getprop ro.build.version.sdk | tr -d '\r')"
[[ "$DEVICE_API" =~ ^[0-9]+$ ]] || die "Could not read the Android API level from device $SERIAL."
(( DEVICE_API >= MIN_SDK )) || die "Device $SERIAL runs Android API $DEVICE_API, but this app requires API $MIN_SDK or later."
DEVICE_MODEL="$("$ADB" -s "$SERIAL" shell getprop ro.product.model | tr -d '\r')"

RUST_TARGET='aarch64-linux-android'
ANDROID_ABI='arm64-v8a'
printf 'Selected device: %s (%s; API %s; %s)\n' "$SERIAL" "${DEVICE_MODEL:-unknown model}" "$DEVICE_API" "$ANDROID_ABI"
printf 'Using Rust target: %s\n' "$RUST_TARGET"

if ! "$RUSTUP" target list --installed | awk -v target="$RUST_TARGET" '$1 == target { found = 1 } END { exit found ? 0 : 1 }'; then
    printf 'Installing missing Rust target %s...\n' "$RUST_TARGET"
    "$RUSTUP" target add "$RUST_TARGET"
fi

ANDROID_BUILD_DIR="$ROOT_DIR/target/dx/$PACKAGE_NAME/debug/android"
ANDROID_GRADLE_DIR="$ANDROID_BUILD_DIR/app"
GENERATED_RES_DIR="$ANDROID_GRADLE_DIR/app/src/main/res"
APK="$ANDROID_BUILD_DIR/app/app/build/outputs/apk/debug/app-debug.apk"

# Remove only Dioxus-generated Android output. This prevents accidentally
# installing a stale native ABI while preserving Cargo caches and app data.
printf 'Removing generated Android output...\n'
rm -rf -- "$ANDROID_BUILD_DIR"

printf 'Building Android debug APK...\n'
(
    cd "$ROOT_DIR"
    "$DX" build --platform android --target "$RUST_TARGET" --locked
)

[[ -f "$APK" ]] || die "Android build completed without producing the expected APK: $APK"
[[ -x "$ANDROID_GRADLE_DIR/gradlew" ]] || die "Generated Android Gradle wrapper not found: $ANDROID_GRADLE_DIR/gradlew"

# Dioxus 0.7.10 stages stock launcher WebP resources. Copy only the committed
# density-specific PNG assets, remove the generated adaptive stock icon, and
# rebuild the final APK. Keeping the launcher icon out of Dioxus.toml avoids
# the generator's duplicate PNG/WebP Android resource collision.
printf 'Applying configured Android launcher icon...\n'
for density in "${ICON_DENSITIES[@]}"; do
    generated_mipmap_dir="$GENERATED_RES_DIR/mipmap-$density"
    mkdir -p -- "$generated_mipmap_dir"
    rm -f -- "$generated_mipmap_dir/ic_launcher.webp"
    cp -- "$ICON_RES_DIR/mipmap-$density/ic_launcher.png" \
        "$generated_mipmap_dir/ic_launcher.png"
done
rm -f -- \
    "$GENERATED_RES_DIR/mipmap-anydpi-v26/ic_launcher.xml" \
    "$GENERATED_RES_DIR/drawable/ic_launcher_background.xml" \
    "$GENERATED_RES_DIR/drawable-v24/ic_launcher_foreground.xml"

# Dioxus derives the Android resource label from the package name and can
# collapse spaces. Override it explicitly so the launcher shows the app name
# exactly as intended.
APP_STRINGS_FILE="$GENERATED_RES_DIR/values/strings.xml"
[[ -f "$APP_STRINGS_FILE" ]] || die "Generated Android strings resource not found: $APP_STRINGS_FILE"
python3 -c 'from pathlib import Path; import sys; p = Path(sys.argv[1]); p.write_text(p.read_text().replace("ChinaTravelApp", "China Travel App"))' "$APP_STRINGS_FILE"

printf 'Rebuilding APK with the configured launcher icon...\n'
(
    cd "$ANDROID_GRADLE_DIR"
    ./gradlew --no-daemon assembleDebug
)

[[ -f "$APK" ]] || die "Gradle completed without producing the expected APK: $APK"

EXPECTED_LIBRARY="lib/$ANDROID_ABI/libmain.so"
APK_ENTRIES="$($UNZIP -Z1 "$APK")"
if ! printf '%s\n' "$APK_ENTRIES" | awk -v expected="$EXPECTED_LIBRARY" '$0 == expected { found = 1 } END { exit found ? 0 : 1 }'; then
    printf 'ERROR: APK does not contain the native library required by %s.\n' "$SERIAL" >&2
    printf 'APK native libraries:\n' >&2
    "$UNZIP" -Z1 "$APK" | awk '/^lib\// { print }' >&2
    exit 1
fi

for density in "${ICON_DENSITIES[@]}"; do
    expected_icon="res/mipmap-$density-v4/ic_launcher.png"
    if ! printf '%s\n' "$APK_ENTRIES" | awk -v expected="$expected_icon" \
        '$0 == expected { found = 1 } END { exit found ? 0 : 1 }'; then
        die "Final APK does not contain the configured $density launcher icon: $expected_icon"
    fi
done
if printf '%s\n' "$APK_ENTRIES" | awk \
    '$0 ~ /ic_launcher\.webp$/ || $0 ~ /mipmap-anydpi-v26\/ic_launcher\.xml$/ { found = 1 } END { exit found ? 0 : 1 }'; then
    die 'Final APK still contains a stock Dioxus launcher icon resource.'
fi

printf 'Installing %s...\n' "$APK"
"$ADB" -s "$SERIAL" install -r "$APK"

if ! "$ADB" -s "$SERIAL" shell pm path "$APPLICATION_ID" | awk '/^package:/ { found = 1 } END { exit found ? 0 : 1 }'; then
    die "Installation reported success, but package $APPLICATION_ID was not present on device $SERIAL."
fi

printf 'Installed successfully on %s (%s).\n' "$SERIAL" "$ANDROID_ABI"
printf 'APK: %s\n' "$APK"
