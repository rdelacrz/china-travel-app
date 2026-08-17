# China Travel Android App Implementation Plan

> **For Hermes:** Use the `subagent-driven-development` skill to implement this plan task-by-task, with specification review followed by code-quality review for each task.

**Goal:** Replace the Dioxus starter with an Android-first, offline China travel app that manages trips, per-trip checklists, and per-trip travel notes/documents backed by local SQLite storage.

**Architecture:** The Dioxus 0.7 UI will call a typed local application/data layer; SQLite will run on a dedicated worker thread in the Android app's private storage. Android-only capabilities—choosing a document, retaining URI access, opening a document, opening a browser URL, and resolving the app-private data directory—will sit behind a narrow platform port implemented by a committed Kotlin `MainActivity.kt` bridge. Dioxus Components supplies accessible primitives; Tailwind supplies app-specific mobile layout and visual styling.

**Tech stack:** Rust, Dioxus 0.7.10, Dioxus Router, Dioxus Components/Dioxus Primitives pinned to a tested Git revision, Tailwind CSS v4 through Dioxus' automatic Tailwind pipeline, `thiserror`, `tokio-rusqlite` + bundled SQLite, `serde`/`serde_json`, `linkify`, Kotlin/Android Storage Access Framework (SAF), Android intents.

---

## 1. Current repository snapshot

The implementation must start from, and preserve unrelated work in, the current mostly-untracked starter tree.

- `Cargo.toml` currently has only Dioxus `0.7.1` with `router` and `fullstack`; default feature is `mobile`.
- Installed tooling observed during planning:
  - `dx 0.7.10`
  - Rust/Cargo `1.96.0`
  - Android SDK platforms 34–36.1 and NDK `30.0.14904198`
  - all four Android Rust targets installed
  - connected device `BYZL25052900312708`, API 30, ABI `armeabi-v7a`
  - Java/JDK and `JAVA_HOME` are currently missing
- Root `tailwind.css` already contains `@import "tailwindcss";`; generated `assets/tailwind.css` is already linked by the starter.
- Starter views: `src/views/home.rs`, `blog.rs`, and `navbar.rs`.
- Starter/test/demo components: `src/components/echo.rs` and `hero.rs`.
- No `.hermes/` directory existed before this plan.

Before changing application files, capture `git status --short --branch` and do not clean/reset the worktree. If commits are authorized, make an initial baseline commit for the existing starter because almost all current files are untracked; otherwise use the commit checkpoints below only as logical review boundaries.

---

## 2. Scope decisions and assumptions

1. **Android only:** no iOS implementation in this phase. Keep non-Android platform behavior explicit as `Unsupported` only where host-side tests need it.
2. **Local backend:** “backend SQLite” means an offline local data/repository layer inside the Android app, not a network server. Remove the unused Dioxus Fullstack/server-function sample.
3. **Trips are the aggregate root:** checklist items and travel documents belong to one trip through `trip_id`.
4. **Minimal trip creation is required to make Home usable:** Home gets a small “Add trip” flow with one required trip name. Trip edit/delete, dates, itinerary, sync, accounts, and cloud backup are out of scope until specified.
5. **No fake/seed trips:** first launch shows an honest empty state. The user creates the first trip.
6. **Attachments are references, not copied blobs:** persist the Android `content://` URI plus display name and MIME type. Do not copy arbitrary files into SQLite or request broad storage permissions.
7. **Downloads is best-effort initial location:** launch `ACTION_OPEN_DOCUMENT` for `*/*`, request the Downloads document root through `DocumentsContract.EXTRA_INITIAL_URI` on supported Android/provider combinations, and keep the full system picker available so the user can navigate to other folders/providers. Some OEM providers may ignore the initial-location hint; this is an Android limitation, not a reason to request unrestricted storage access.
8. **Persisted access:** call `takePersistableUriPermission` using only the read flag actually granted by the picker. Cancellation is a normal outcome. A provider that refuses persistable access returns a precise, user-visible unsupported-provider error rather than silently saving a URI that will fail after reboot.
9. **Document links:** detect and render `http://`, `https://`, and `www.` URLs in descriptions. Normalize `www.` to HTTPS. Never render stored descriptions as raw HTML and never launch non-HTTP(S) schemes from description text.
10. **Development package identity:** use a clearly marked development identifier such as `com.rdelacruz.chinatravel` only after confirming it with the user; lock the final package ID before release/signing.
11. **Minimum Android version:** use API 26 as the initial minimum so `EXTRA_INITIAL_URI` is available. Target/compile against an installed current SDK (35 unless implementation evidence requires 36). Confirm this product decision before release.
12. **No document delete flow in this slice:** the request specifies create/edit/view/open, not delete. Do not invent it.
13. **China-specific information beyond the requested summary/checklists/documents is deferred:** do not add speculative visa, currency, map, or network content without requirements.

---

## 3. Target dependency and feature shape

Update `Cargo.toml` so crate and CLI versions are compatible and builds are reproducible.

```toml
[dependencies]
dioxus = { version = "=0.7.10", features = ["router"] }
dioxus-primitives = { git = "https://github.com/DioxusLabs/dioxus-components", rev = "bf007c15d0cf4d04d3181cc46cf12325aa773955", features = ["router"] }
dioxus-icons = "0.1.0"
linkify = "0.11.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2.0.20"
tokio-rusqlite = { version = "0.7.0", features = ["bundled"] }
url = "2"

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

[features]
default = ["mobile"]
mobile = ["dioxus/mobile"]

[[bin]]
name = "china-travel-app"
path = "src/main.rs"
required-features = ["mobile"]
```

Notes:

- `tokio-rusqlite` runs each connection on its own worker thread and re-exports its compatible `rusqlite`; do not add a mismatched direct `rusqlite` version.
- The `bundled` feature avoids relying on an OEM/system SQLite development library during Android cross-compilation.
- Remove `fullstack`, `server`, and the starter server function. Product web/desktop features are not supported in this phase; library tests compile with `--no-default-features`.
- Re-query registry versions during implementation, but do not silently upgrade Dioxus or the pinned Dioxus Components revision while implementing this plan. A dependency update is a separate reviewed change.

Generate first-party components with the CLI instead of hand-copying them:

```bash
dx components add \
  --git https://github.com/DioxusLabs/dioxus-components \
  --rev bf007c15d0cf4d04d3181cc46cf12325aa773955 \
  alert_dialog button card checkbox input label sheet textarea toast
```

Review every generated change. Keep the accessible Dioxus primitive behavior and generated baseline styles; use Tailwind utilities for all app-specific layout/spacing/colors/responsiveness. Pin the resulting `dioxus-primitives` dependency to the same revision if the CLI emits an unpinned Git dependency.

---

## 4. Target modules and files

### Create

```text
migrations/
└── 0001_initial.sql

android/
└── MainActivity.kt

src/
├── lib.rs
├── app.rs
├── error.rs
├── state.rs
├── domain/
│   ├── mod.rs
│   ├── trip.rs
│   ├── checklist.rs
│   └── document.rs
├── db/
│   ├── mod.rs
│   ├── migrations.rs
│   ├── trips.rs
│   ├── checklist.rs
│   └── documents.rs
├── platform/
│   ├── mod.rs
│   ├── protocol.rs
│   ├── android.rs
│   └── fake.rs                 # cfg(test) host adapter
├── components/
│   ├── app_shell.rs
│   ├── trip_pane.rs
│   ├── checklist_item_pane.rs
│   ├── document_pane.rs
│   ├── document_sheet.rs
│   ├── linked_text.rs
│   └── <generated component directories/files from dx>
└── views/
    ├── mod.rs
    ├── home.rs
    ├── checklist.rs
    └── documentation.rs

tests/
├── database.rs
├── checklist_workflow.rs
├── document_workflow.rs
└── bridge_protocol.rs
```

Only keep a shared component when it owns repeated structure **and behavior**. For example, `ChecklistItemPane`, `DocumentPane`, `DocumentSheet`, and `AppShell` are justified; do not introduce zero-value wrappers around a native `<li>`, `<section>`, or `<button>`.

### Modify

- `Cargo.toml`
- `Cargo.lock`
- `Dioxus.toml`
- `README.md`
- `tailwind.css`
- generated `assets/tailwind.css`
- `src/main.rs` — launcher only
- `src/components/mod.rs`
- `src/views/mod.rs`

### Delete starter/demo files

- `src/views/blog.rs`
- `src/views/navbar.rs`
- existing starter contents of `src/views/home.rs` (rewrite the file)
- `src/components/echo.rs`
- `src/components/hero.rs`
- `assets/header.svg`
- `assets/styling/blog.css`
- `assets/styling/echo.css`
- `assets/styling/navbar.css`
- `assets/styling/main.css` after equivalent global styling is in Tailwind

Remove empty starter styling directories after their files are gone. Keep `favicon.ico` until an app icon is supplied; do not fabricate branding assets.

---

## 5. Application routes and state ownership

Use exactly the three requested view modules:

```rust
#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(AppShell)]
        #[route("/")]
        Home {},
        #[route("/trips/:trip_id/checklist")]
        Checklist { trip_id: i64 },
        #[route("/trips/:trip_id/documentation")]
        Documentation { trip_id: i64 },
}
```

`src/main.rs` launches the library root with Android WebView configuration. `src/app.rs` owns startup and routing. Keep conditional hook use out of `App`: load `AppContext` in a resource, render loading/error states, and render an `AppReady` child that provides the initialized context.

```text
AppContext
├── Database                 cloned tokio-rusqlite connection handle
├── Arc<dyn PlatformPort>    Android adapter or test fake
└── Signal<u64> revision     increment after successful writes
```

Views read `revision` in their resources so returning to Home refreshes aggregate counts. Mutations update visible state only after SQLite succeeds, or use an explicitly tested optimistic update with rollback. Never leave UI state claiming a write succeeded when SQLite failed.

`AppShell` provides mobile-safe page chrome and navigation. Trip cards link directly to the trip's Checklist and Documentation routes; detail pages show Home, Checklist, and Documentation navigation for the current `trip_id`.

---

## 6. SQLite schema and repository contract

Use one committed migration and a migration runner based on `PRAGMA user_version`. Initialization must use `BEGIN IMMEDIATE`, enable foreign keys before schema work, apply only forward migrations, set `user_version` in the same transaction, and be safe to rerun.

`migrations/0001_initial.sql`:

```sql
CREATE TABLE trips (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL CHECK (length(trim(name)) > 0),
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE checklist_items (
    id          INTEGER PRIMARY KEY,
    trip_id     INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    text        TEXT NOT NULL CHECK (length(trim(text)) > 0),
    is_checked  INTEGER NOT NULL DEFAULT 0 CHECK (is_checked IN (0, 1)),
    sort_order  INTEGER NOT NULL CHECK (sort_order >= 0),
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (trip_id, sort_order)
);

CREATE INDEX idx_checklist_items_trip
    ON checklist_items(trip_id, sort_order, id);

CREATE TABLE travel_documents (
    id                       INTEGER PRIMARY KEY,
    trip_id                  INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    name                     TEXT NOT NULL CHECK (length(trim(name)) > 0),
    description              TEXT NOT NULL DEFAULT '',
    attachment_uri           TEXT,
    attachment_display_name  TEXT,
    attachment_mime_type     TEXT,
    created_at               INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at               INTEGER NOT NULL DEFAULT (unixepoch()),
    CHECK (attachment_uri IS NULL OR length(trim(attachment_uri)) > 0)
);

CREATE INDEX idx_travel_documents_trip
    ON travel_documents(trip_id, updated_at DESC, id DESC);
```

Connection initialization:

```sql
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
```

Treat fresh creation and upgrade of an existing database as different tests. Never edit an already-released migration; add `0002_*.sql` later.

Required repository methods:

```text
Database::open(path) -> Result<Database, DbError>
Database::open_in_memory() -> Result<Database, DbError>
Database::migrate() -> Result<(), DbError>

Trips:
create_trip(name)
get_trip(id)
list_trip_overviews()  # checklist total/done/outstanding + document count

Checklist:
list_checklist(trip_id)
create_checklist_item(trip_id, text)
rename_checklist_item(item_id, text)
set_checklist_item_checked(item_id, checked)
delete_checklist_item(item_id)

Documents:
list_documents(trip_id)
get_document(document_id)
create_document(NewTravelDocument)
update_document(UpdateTravelDocument)
```

Every update/delete checks `changed_rows == 1`; otherwise return typed `NotFound`. Trim names/text at the boundary. Reject blank values before SQL. Use a transaction when computing and inserting the next checklist `sort_order`.

Do not store raw file bytes, secrets, or unrestricted filesystem paths in SQLite.

---

## 7. Error model (`thiserror`)

Define narrow error layers and preserve their sources:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("trip name cannot be blank")]
    BlankTripName,
    #[error("checklist item cannot be blank")]
    BlankChecklistText,
    #[error("document name cannot be blank")]
    BlankDocumentName,
    #[error("only HTTP and HTTPS links can be opened")]
    UnsupportedUrlScheme,
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database worker closed")]
    WorkerClosed,
    #[error("{entity} {id} was not found")]
    NotFound { entity: &'static str, id: i64 },
    #[error("SQLite operation failed")]
    Sqlite(#[from] tokio_rusqlite::rusqlite::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("this capability is unsupported on the current platform")]
    Unsupported,
    #[error("the selected provider did not grant persistent read access")]
    PersistablePermissionDenied,
    #[error("access to the attached file is no longer available")]
    AttachmentUnavailable,
    #[error("no installed app can open this content")]
    NoActivityHandler,
    #[error("native bridge protocol error: {0}")]
    Protocol(String),
    #[error("native operation failed: {0}")]
    Native(String),
}
```

Add an `AppError` wrapper only where orchestration genuinely needs one. Do not use broad `catch (Exception)`/`unwrap()` to flatten failures. In Kotlin, map documented `ActivityNotFoundException`, `SecurityException` from URI grants/access, cancellation, malformed requests, and unsupported API/provider conditions separately; let unexpected programming failures surface in debug builds.

Picker cancellation is **not** `PlatformError`:

```rust
pub enum PickDocumentOutcome {
    Selected(AttachmentRef),
    Cancelled,
}
```

User-facing toasts should be concise and should not expose database paths or full private document URIs. Debug logging may include operation/request IDs and error kinds, but not description contents or attachment URIs.

---

## 8. Android platform bridge contract

Prefer the installed, verified Dioxus/Wry customization point over the currently open/unstable `manganis::ffi` path:

- `Dioxus.toml [application].android_main_activity = "android/MainActivity.kt"`
- custom file package remains `dev.dioxus.main`
- alias the application `BuildConfig` using the final Android identifier
- `MainActivity : WryActivity()`
- register the narrow bridge in `onWebViewCreate(WebView)`
- never edit `target/dx/**` generated Android files

Use versioned, correlated JSON envelopes. Only a fixed base64-encoded JSON payload may be interpolated into `evaluateJavascript`; never interpolate raw names, URIs, MIME types, or URLs into executable JavaScript.

```rust
#[derive(Serialize, Deserialize)]
struct NativeRequest {
    version: u8,
    request_id: String,
    operation: NativeOperation,
}

enum NativeOperation {
    AppDataDirectory,
    PickDocument { prefer_downloads: bool },
    OpenDocument { uri: String, mime_type: Option<String> },
    OpenUrl { url: String },
    ReleaseReadPermission { uri: String },
}

struct AttachmentRef {
    uri: String,
    display_name: Option<String>,
    mime_type: Option<String>,
}
```

Bridge rules:

1. Resolve every request exactly once; reject or serialize concurrent picker requests.
2. Register `ActivityResultContracts.StartActivityForResult` while the Activity is in a valid lifecycle state.
3. Picker intent:
   - `Intent.ACTION_OPEN_DOCUMENT`
   - `Intent.CATEGORY_OPENABLE`
   - MIME `*/*`
   - read + persistable URI flags
   - best-effort Downloads `EXTRA_INITIAL_URI`
4. On success:
   - read only the granted flags from the result intent;
   - call `takePersistableUriPermission(uri, grantedReadFlag)`;
   - query `OpenableColumns.DISPLAY_NAME` and `ContentResolver.getType(uri)`;
   - return URI + optional metadata.
5. On document open: use `ACTION_VIEW`, exact stored MIME type or `*/*`, and `FLAG_GRANT_READ_URI_PERMISSION`.
6. On URL open: validate HTTP(S) in Rust and again in Kotlin, then use `ACTION_VIEW`.
7. App data path: return `filesDir` (or a dedicated child directory) and create `china-travel.sqlite3` there. Never use Downloads/shared storage for the database.
8. Keep the Dioxus WebView on trusted packaged app content. Description links call `OpenUrl`; they never navigate the app WebView to untrusted pages.
9. Keep the component that owns an async picker/delete operation mounted until completion. Prefer a stable view-owned task/channel; Dioxus cancels spawned work when its owner unmounts.
10. When replacing an attachment, acquire the new persisted grant first, commit the database update, then release the old grant. If the database update fails, release the newly acquired grant. Do not release an unchanged URI.

No `READ_EXTERNAL_STORAGE`, `WRITE_EXTERNAL_STORAGE`, or `MANAGE_EXTERNAL_STORAGE` permission is needed or allowed for this feature.

---

## 9. UI behavior and acceptance details

### Home (`src/views/home.rs`)

- Top overview card contains app purpose plus aggregate metrics:
  - number of trips;
  - outstanding checklist items;
  - completed checklist items;
  - saved documents.
- Show persisted trip panes below, with trip name and per-trip counts.
- Each trip pane has clear actions to enter Checklist or Documentation.
- Honest loading, empty, and retry states.
- Minimal “Add trip” interaction:
  - required name only;
  - trim before insert;
  - blank input stays open with validation;
  - update list and aggregate summary only after SQLite succeeds.

### Checklist (`src/views/checklist.rs`)

- Load the route trip or show a typed not-found state.
- Render checklist items in database `sort_order`.
- Each pane contains:
  - text as the flexible main area;
  - checkbox on the right;
  - X icon delete button on the right with an accessible label.
- Tapping text switches only that row into edit mode.
- Edit input uses `enterkeyhint="go"`, selects/focuses appropriately, and commits on focus loss or `Key::Enter`.
- Deduplicate Enter followed by blur: one user edit must cause at most one SQL update.
- Trim text; do not persist blank text. Keep the row editable and show validation when blank.
- Checkbox state is persisted immediately; disable repeat input while its write is active and roll back any optimistic UI state on failure.
- X opens generated Dioxus `AlertDialog` with item-specific text, Cancel, and destructive Delete action.
- Confirmed deletion runs from the stable Checklist view, removes the row only after SQLite success, and reports failure through a toast.
- Sticky “Add item” button remains at the bottom above navigation/safe-area insets.
- Add creates a client-side draft row, focuses it, and inserts only when the first nonblank value is committed. An untouched/blank draft can be discarded without writing junk data.

### Documentation (`src/views/documentation.rs`)

- Load documents newest-updated first.
- Sticky “Add document” button opens the generated Dioxus `Sheet` from the **right**.
- Add/edit sheet fields:
  - Document name (`Input`, required)
  - Description (`Textarea`, optional)
  - Attach/change file button (`Button`)
  - current attachment display name and a remove-from-form control when present
  - Save and Cancel fixed in `SheetFooter`
- Picker cancellation leaves the form and current attachment unchanged.
- Cancel closes the sheet and discards unsaved form changes; it does not release the currently persisted attachment for an existing record.
- Save is disabled while saving and rejects a blank trimmed name.
- Save inserts or updates SQLite first, then closes the sheet and updates the list.
- Each saved document pane shows:
  - full title;
  - truncated description in collapsed mode;
  - right-side View and Edit icon buttons;
  - right-side Open File icon button only when `attachment_uri.is_some()`.
- View toggles the same pane between collapsed and full description.
- Expanded description preserves line breaks and renders detected HTTP(S) links as safe clickable nodes. URL clicks launch the external browser through `PlatformPort`; never use `dangerous_inner_html`.
- Edit opens the same right-side sheet prefilled with the existing record and attachment metadata.
- Open File launches the Android handler for the stored URI. Missing/revoked access and no-handler cases produce distinct messages and do not crash.

### Shared mobile behavior

- Use `min-h-dvh`, safe-area padding, and touch targets of at least 48dp.
- Keep primary actions reachable above the software keyboard and bottom navigation.
- Icon-only buttons require `aria-label`/accessible text and visible focus states.
- Generated Sheet and AlertDialog retain their focus trap, Escape/backdrop behavior, labels, and modal semantics.
- Respect Android back: close Sheet/Dialog before leaving the route.
- Use loading/busy states to prevent duplicate mutations.

---

## 10. Tailwind and component styling

Configure Dioxus' automatic Tailwind pipeline explicitly:

```toml
[application]
tailwind_input = "tailwind.css"
tailwind_output = "assets/tailwind.css"
android_main_activity = "android/MainActivity.kt"

[android]
min_sdk = 26
target_sdk = 35
compile_sdk = 35
```

The exact bundle identifier belongs in the current Dioxus bundle/application config after confirmation.

Use explicit source discovery in root `tailwind.css` and exclude generated output from scanning itself:

```css
@import "tailwindcss" source(none);
@source "./src/**/*.{rs,html}";
@source not "./assets/tailwind.css";
```

Add app tokens/base rules in this input file (China-inspired red accent, neutral readable surfaces, danger/success states, safe areas), not in ad-hoc per-view CSS. Keep generated Dioxus component CSS and `assets/dx-components-theme.css` as component-owned baseline assets. Remove all starter CSS.

Mount exactly one Tailwind stylesheet plus the Dioxus Components global theme. For Android, configure `dioxus::mobile::Config` with the same packaged stylesheet links and an app-matching background color before launch to avoid a white first-paint flash.

Verification:

1. Remove only `assets/tailwind.css`, run the canonical Dioxus/Tailwind build, and record SHA-256/size.
2. Repeat from absent output and require an identical hash/size.
3. Confirm selectors used in RSX (including state/focus/disabled and arbitrary safe-area utilities) exist.
4. Confirm the Android bundle contains and loads the generated stylesheet and component theme.

---

## 11. Task-by-task implementation sequence

### Task 1: Establish a reproducible Android build prerequisite

**Objective:** Make the existing starter capable of an Android build before feature work.

**Files:** No application code initially; update `README.md` only after commands are proven.

1. Capture `git status`, installed Rust targets, `dx --version`, SDK/NDK paths, device ABI, and API level.
2. Install/configure the JDK version required by the generated Gradle wrapper (expected JDK 17); set `JAVA_HOME` only after verifying `java -version` and `./gradlew -version` compatibility.
3. Export `ANDROID_HOME`, `ANDROID_SDK_ROOT`, `NDK_HOME`, and `ANDROID_NDK_HOME`; put Java and `platform-tools` on `PATH` in a repeatable script/documented shell block.
4. Run `dx doctor`.
5. Build the starter explicitly for the connected device ABI:

```bash
dx build --android --target armv7-linux-androideabi
```

6. Record the exact successful environment in `README.md`; do not encode transient absolute generated paths.

**Expected:** The build reaches APK/native-library generation for `armeabi-v7a`. Any Dioxus/CLI version warning is handled separately from build failure.

**Commit checkpoint (if authorized):** `docs: document Android build prerequisites`

### Task 2: Remove the starter and create the application skeleton

**Objective:** Leave only the three requested views and a testable library/root route structure.

**Files:** Delete starter files listed in §4; create `src/lib.rs`, `src/app.rs`; rewrite `src/main.rs`, `src/views/mod.rs`, `src/components/mod.rs`.

1. Write route serialization/display tests for Home, Checklist, and Documentation.
2. Delete Blog/Navbar/Echo/Hero and their sample assets/styles.
3. Move `App` and `Route` to library modules; make `main.rs` launcher-only.
4. Add temporary placeholder implementations only for the three requested view components so the route skeleton compiles.
5. Run:

```bash
cargo test --no-default-features
cargo check --no-default-features
```

**Expected:** No references to Blog, Echo, Hero, server functions, or starter CSS remain.

**Commit checkpoint:** `refactor: replace Dioxus starter routes`

### Task 3: Generate Dioxus Components and wire Tailwind

**Objective:** Establish the component/style system before feature UI.

**Files:** `Cargo.toml`, `Cargo.lock`, `tailwind.css`, `assets/tailwind.css`, `src/components/**`, `assets/dx-components-theme.css`, `Dioxus.toml`, `src/main.rs`/`src/app.rs`.

1. Run the pinned `dx components add` command in §3.
2. Review generated source/dependencies/assets; pin `dioxus-primitives` to the same Git SHA.
3. Add explicit Tailwind source directives and app theme tokens.
4. Mount Tailwind and Dioxus component theme once.
5. Add a small component showcase behind tests or a temporary local route only while verifying Sheet, AlertDialog, Checkbox, Input, Textarea, Toast, and Button; remove the showcase before committing.
6. Run two clean Tailwind builds and compare SHA-256.
7. Run component compile checks.

**Expected:** Generated components compile on Dioxus 0.7.10; starter test/demo components are gone; app-specific styling is Tailwind-only.

**Commit checkpoint:** `feat: add Dioxus component and Tailwind foundation`

### Task 4: Define domain models, validation, and errors

**Objective:** Make business/data contracts independent of views and Android.

**Files:** `src/domain/**`, `src/error.rs`.

1. Write failing tests for blank/trimmed trip names, checklist text, and document names.
2. Add owned models: `Trip`, `TripOverview`, `ChecklistItem`, `TravelDocument`, `NewTravelDocument`, `UpdateTravelDocument`, and `AttachmentRef`.
3. Implement validation constructors/update methods and error enums from §7.
4. Test that descriptions may be blank and attachments are optional/coherent.
5. Run focused tests, then all host tests.

**Commit checkpoint:** `feat: define travel domain and typed errors`

### Task 5: Define and test the platform protocol

**Objective:** Freeze a narrow native-capability contract before Kotlin integration.

**Files:** `src/platform/mod.rs`, `protocol.rs`, `fake.rs`, `tests/bridge_protocol.rs`.

1. Write round-trip JSON tests for every request/success/failure/cancel envelope.
2. Test unknown protocol versions, request IDs, operation names, malformed payloads, and unsupported URL schemes.
3. Define `PlatformPort` operations: app data directory, pick document, open document, open URL, release read grant.
4. Build a deterministic fake adapter for host workflow tests.
5. Require cancellation to deserialize as `PickDocumentOutcome::Cancelled`, not an error.

**Commit checkpoint:** `feat: define Android bridge protocol`

### Task 6: Prove and implement the committed Android host bridge

**Objective:** Verify the native customization hook on a real device before database/document UI depends on it.

**Files:** `android/MainActivity.kt`, `Dioxus.toml`, `src/platform/android.rs`, `src/main.rs`.

1. Add the smallest custom `MainActivity : WryActivity()` and prove `dx` selects it; do not edit generated Android output.
2. Register the bridge in `onWebViewCreate` and prove one `AppDataDirectory` request round-trips.
3. Add picker launch/result handling, persisted read permission, metadata lookup, open-file intent, open-URL intent, and grant release.
4. Ensure callbacks run on the main thread and each request resolves once.
5. Exercise success, cancellation, malformed request, concurrent picker request, no handler, and denied/revoked access on the connected API-30 device.
6. Rotate/background/return during the picker and verify no stale pending request or crash.
7. Inspect logs for URI/description leakage.
8. Build and bundle with the explicit `armv7-linux-androideabi` target.

**Expected:** The actual system picker opens (not merely the button handler), starts in Downloads when the provider honors the hint, and still exposes other locations.

**Commit checkpoint:** `feat: add Android document and intent bridge`

### Task 7: Implement SQLite initialization and migration tests

**Objective:** Provide a durable, non-blocking local database.

**Files:** `migrations/0001_initial.sql`, `src/db/mod.rs`, `migrations.rs`, `tests/database.rs`, `src/state.rs`, `src/app.rs`.

1. Write failing tests for fresh migration, second initialization, `user_version`, foreign keys, indexes, CHECK constraints, and cascade deletion.
2. Open SQLite through `tokio-rusqlite` on its worker thread.
3. Implement transactional migration application and connection PRAGMAs.
4. Initialize the Android database under the bridge-provided private data directory.
5. Add startup loading/error/retry UI; never render Router with an uninitialized database.
6. Run tests against both in-memory and temporary-file databases.

**Commit checkpoint:** `feat: initialize local SQLite storage`

### Task 8: Implement trips and Home overview

**Objective:** Make the app usable from an empty first launch and provide aggregate summary counts.

**Files:** `src/db/trips.rs`, `src/views/home.rs`, `src/components/trip_pane.rs`, `tests/database.rs` or `tests/trip_workflow.rs`.

1. Write repository tests for blank-name rejection, create/list ordering, not found, and aggregate counts with multiple trips.
2. Implement `create_trip`, `get_trip`, and `list_trip_overviews` with one aggregate query rather than N+1 per trip.
3. Build Home loading/empty/error/list states and top summary.
4. Add the minimal name-only Add Trip flow.
5. Add route links from each `TripPane` to Checklist and Documentation.
6. Verify process restart preserves trips.

**Commit checkpoint:** `feat: add trip overview home`

### Task 9: Implement checklist persistence

**Objective:** Complete checklist CRUD before wiring row event complexity.

**Files:** `src/db/checklist.rs`, `tests/checklist_workflow.rs`.

1. Write failing tests for ordered insert, rename, checked toggle, delete, blank text, wrong IDs, cross-trip isolation, and trip cascade.
2. Implement repository methods with changed-row checks.
3. Allocate `sort_order` transactionally.
4. Test concurrent/repeated calls on the single worker and ensure no duplicate sort positions.

**Commit checkpoint:** `feat: persist trip checklist items`

### Task 10: Build the Checklist view and item pane

**Objective:** Implement all requested mobile checklist interactions.

**Files:** `src/views/checklist.rs`, `src/components/checklist_item_pane.rs`, `src/components/app_shell.rs`.

1. Extract a testable edit-state reducer/command layer and write tests for add draft, click-to-edit, Enter, blur, blank rejection, and Enter+blur de-duplication.
2. Render ordered rows with right-side Checkbox and X icon.
3. Commit rename on blur or mobile Go/Enter.
4. Add stable-parent async commands for toggle and deletion.
5. Use generated AlertDialog for confirmation.
6. Add the sticky bottom Add Item button and draft-row autofocus.
7. Verify database failure retains/reverts the correct visible state and shows a toast.
8. Exercise keyboard, focus, checkbox persistence, Cancel delete, Confirm delete, back navigation, and process restart on Android.

**Commit checkpoint:** `feat: add editable travel checklist`

### Task 11: Implement document persistence and attachment transitions

**Objective:** Store document metadata and define safe attachment replacement behavior.

**Files:** `src/db/documents.rs`, `tests/document_workflow.rs`.

1. Write failing tests for create with/without attachment, edit, attachment retain/replace/remove, blank name, ordering, not found, cross-trip isolation, and trip cascade.
2. Implement create/list/get/update methods.
3. Add orchestration tests with `FakePlatformPort` for:
   - picker canceled;
   - new grant + successful DB save;
   - new grant + failed DB save releases new grant;
   - replace success releases old grant only after commit;
   - unchanged attachment releases nothing.
4. Ensure no document row is changed when native selection fails.

**Commit checkpoint:** `feat: persist travel documents`

### Task 12: Build Add/Edit Document Sheet

**Objective:** Implement the right-side create/edit form with optional Android attachment.

**Files:** `src/views/documentation.rs`, `src/components/document_sheet.rs`.

1. Write form-state tests for Add defaults, Edit prefill, validation, Cancel, Save busy state, and attachment retain/change/remove.
2. Keep form state and picker task owned by the stable Documentation view so Sheet internals do not cancel the native request.
3. Render generated Sheet with explicit right side, accessible title/description, Input, Textarea, attachment controls, Save, and Cancel.
4. Wire picker outcome without closing the Sheet.
5. Save to SQLite, update list, then close/reset the form.
6. Verify optional attachment: saving without one succeeds.
7. Exercise picker Downloads start, other-folder navigation, arbitrary MIME types, cancellation, rotation/background return, and reboot persistence.

**Commit checkpoint:** `feat: add document sheet workflow`

### Task 13: Build document panes, expanded links, and open actions

**Objective:** Complete requested display/view/edit/open behavior.

**Files:** `src/components/document_pane.rs`, `src/components/linked_text.rs`, `src/views/documentation.rs`.

1. Write link segmentation tests for multiple URLs, punctuation, multiline text, `www.`, unsupported schemes, and plain text.
2. Render title + truncated description + right-side View/Edit icons.
3. Render Open File icon only for attached records.
4. Toggle the same pane to expanded full description.
5. Render safe text/link segments without HTML injection.
6. Launch HTTP(S) links externally and attached URIs through the platform port.
7. Test no-handler and revoked-URI feedback.

**Commit checkpoint:** `feat: add document viewing and external open actions`

### Task 14: Mobile polish, accessibility, and error recovery

**Objective:** Make all three views reliable on a phone rather than merely compiling.

**Files:** `tailwind.css`, affected views/components, `src/state.rs`, `README.md`.

1. Add safe-area, dynamic viewport, sticky-action, keyboard, spacing, color, and 48dp touch-target utilities.
2. Add labels/focus states for all icon-only controls.
3. Verify AlertDialog and Sheet keyboard/back/focus semantics remain intact after styling.
4. Add bounded loading/disabled states to prevent duplicate mutations.
5. Add startup and per-view retry controls.
6. Test long titles/descriptions, hundreds of checklist rows, empty descriptions, missing MIME types, and large accessibility font settings.
7. Remove temporary showcase/debug UI and sensitive logs.

**Commit checkpoint:** `fix: harden mobile travel workflows`

### Task 15: Final quality and Android acceptance gate

**Objective:** Produce evidence that the actual APK, database, and native integrations work.

1. Host checks:

```bash
cargo fmt --check
cargo clippy --no-default-features --all-targets -- -D warnings
cargo test --no-default-features
```

2. Android compile/package for the actual connected ABI:

```bash
dx build --android --target armv7-linux-androideabi --locked
dx bundle --android --package-types apk --target armv7-linux-androideabi --locked
```

3. Inspect the APK rather than only the Gradle directory:

```bash
unzip -Z1 "$APK" | grep '^lib/'
```

Require an `armeabi-v7a` native library before installation.

4. Install and confirm package path on the same serial:

```bash
adb -s BYZL25052900312708 install -r "$APK"
adb -s BYZL25052900312708 shell pm path <confirmed-package-id>
```

5. Run the full manual/device matrix in §12.
6. Inspect `adb logcat` for panics, bridge errors, SQLite errors, leaked URIs/descriptions, and lifecycle faults.
7. Re-run host tests after any device-found fix.
8. Recheck `git status`, diff, generated CSS determinism, deleted starter references, and production-file sizes.
9. Store concise machine-readable execution evidence under `.hermes/testing/` during implementation; do not commit raw private document names/URIs or device logs containing personal data.
10. Update README with proven build/run/test commands and current limitations.

**Commit checkpoint:** `test: verify Android travel app workflows`

---

## 12. Required Android acceptance matrix

### Home/trips

- Fresh install shows zero-count summary and useful empty state.
- Create two named trips; both survive process kill/relaunch.
- Aggregate counts change after checklist/document writes and remain trip-isolated.
- Invalid/missing route trip ID shows a recoverable not-found view.

### Checklist

- Add Item creates a focused draft at the bottom.
- Blank draft does not write to SQLite.
- Blur commits once.
- mobile Go/Enter commits once even if blur follows.
- Click text re-enters edit mode.
- Check/uncheck persists after relaunch.
- X → Cancel makes no change.
- X → Delete removes exactly one item after confirmation.
- Write failure does not leave false checked/text/deleted UI state.

### Documents

- Save name + description without attachment; no Open File icon is rendered.
- Add Document opens a right-side Sheet.
- Picker opens at Downloads when supported and allows navigation to another folder/provider.
- Pick at least a PDF, image, and unknown/other MIME type.
- Picker cancellation preserves the form.
- Save attached document; Open File launches a compatible Android app.
- Kill/relaunch and reboot; persisted URI still opens.
- Edit name/description while retaining attachment.
- Replace attachment; new file opens and old grant is released only after successful save.
- Remove attachment in Edit; Open File icon disappears after save.
- Provider access revoked/file removed produces typed feedback, not a crash.
- No installed handler produces typed feedback.
- View expands full multiline description.
- Multiple HTTP(S)/`www.` links are clickable and open the external browser.
- `javascript:`, `file:`, malformed text, and stored `<script>` remain inert escaped text.

### Lifecycle/accessibility

- Android Back closes AlertDialog/Sheet before route navigation.
- Rotate/background/return while picker is open.
- No duplicate callbacks or stuck busy states.
- TalkBack labels announce checkbox, delete, view, edit, attach, open-file, Save, and Cancel purpose.
- Touch targets remain usable and sticky buttons stay above safe areas/keyboard.

### Packaging/data safety

- APK includes the connected device ABI.
- No broad external-storage permission exists in merged manifest.
- SQLite database is in app-private storage.
- No raw attachment data is stored in SQLite.
- Logs do not expose descriptions, attachment URIs, or document names.

---

## 13. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Java is missing, blocking Gradle before feature work | Resolve JDK/`JAVA_HOME` in Task 1 and prove a starter Android build first. |
| Connected phone is 32-bit ARM even though many Android defaults target arm64 | Always pass `--target armv7-linux-androideabi`, inspect APK `lib/`, and install on the same serial. Add other ABIs only when a target device/release requirement exists. |
| Dioxus Components registry is Git-based and moving | Generate with `--rev bf007…` and pin `dioxus-primitives` to the same revision. Upgrade separately. |
| `manganis::ffi` currently has an open minimal-example issue | Use the verified custom `MainActivity.kt`/Wry hook; keep the Rust port narrow so a future bridge can replace it. |
| OEM file provider ignores Downloads initial URI | Treat `EXTRA_INITIAL_URI` as a hint, verify on target devices, retain full picker navigation, and document the OS limitation. |
| Provider does not grant persistable permission or later revokes access | Fail save precisely when persistence cannot be obtained; handle later `SecurityException`/missing content as `AttachmentUnavailable`. |
| Dioxus task is canceled when Sheet/Dialog child unmounts | Own picker/delete tasks in a stable view or root command runner and keep modals mounted until callback handoff. |
| Enter then blur causes duplicate checklist writes | Centralize an idempotent per-row commit command keyed by current draft/saved value and test exact SQL call count. |
| SQLite work blocks UI | Route all database work through `tokio-rusqlite`'s dedicated worker thread. |
| Changing an embedded `CREATE TABLE IF NOT EXISTS` fails to upgrade existing installs | Use ordered forward migrations + `user_version`; test fresh and legacy-forward paths separately. |
| External description links navigate the trusted WebView | Render escaped segments and route only validated HTTP(S) URLs through Android `ACTION_VIEW`. |
| Package ID changes break stored permissions/release identity | Confirm and freeze bundle identifier before release/signing; treat changes as migration/reinstall events. |

---

## 14. Definition of done

The implementation is complete only when:

- the starter Blog/Navbar/Echo/Hero views/components and styles are removed;
- `src/views/` contains only `mod.rs`, `home.rs`, `checklist.rs`, and `documentation.rs`;
- `src/components/` contains CLI-generated Dioxus components plus only behaviorful shared app components;
- Home lists persisted trips and accurate summary counts;
- all requested checklist edit/toggle/add/delete-confirmation behavior persists to SQLite;
- all requested document add/edit/view/optional-attachment/open behavior works;
- description links safely open in the external browser;
- Android picker access survives relaunch/reboot where the provider grants persistence;
- errors are typed with `thiserror` and cancellation is not flattened into failure;
- host tests, clippy, formatting, Android build, APK ABI inspection, installation, and the real-device matrix all pass;
- no generated Android project under `target/` was hand-edited;
- README contains only commands and limitations verified during real execution.
