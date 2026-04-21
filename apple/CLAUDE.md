# CLAUDE.md

Guidance for Claude Code when working with **Photonic** (iOS photo backup app).

## Purpose

Photonic backs up photos from iOS to a Photonic server. The app is written in
Swift with SwiftUI, uses **OAuth2 with PKCE** for auth, and a **type-safe API
client generated from OpenAPI**. This doc explains how to keep the codebase
clean, testable, secure, and aligned with **Hexagonal Architecture** and
**DDD**.

---

## Tech Stack (at a glance)

- **Language/UI**: Swift + SwiftUI
- **Concurrency**: Swift Concurrency (actors, async/await)
- **API**: `swift-openapi-generator` → Generated client
- **Auth**: OAuth2/OIDC + PKCE, JWT parsing
- **Persistence**: SwiftData (for local app state), Keychain (tokens)
- **Hot reload**: Inject (dev only)
- **Tooling**: SwiftLint, SwiftFormat, XCTests, XCUITests

---

## Project Structure (Hexagonal + DDD)

Refactor/organize into the following top-level groups. Keep generated code
separated and never import Infra directly from Domain/Application.

```
Photonic/
├── App/                                # iOS app target (composition root)
│   ├── PhotonicApp.swift               # Entry point, DI wiring, Scene
│   └── CompositionRoot.swift           # Build adapters & inject use cases
│
├── Domain/                             # DDD Core (pure Swift)
│   ├── Entities/                       # e.g., MediaItem, Album, UserAccount
│   ├── ValueObjects/                   # Token, ServerURL, MediaHash
│   ├── Services/                       # Domain services (pure logic)
│   ├── Repositories/                   # Protocols only (ports)
│   └── Errors/                         # DomainError, AuthError
│
├── Application/                        # Services (ports & orchestrations)
│   ├── Services/                       # Application-level services if needed
│   ├── DTOs/                           # App-layer models if needed
│   └── Interfaces/                     # Application ports (protocols)
│
├── Infrastructure/                     # Adapters (to frameworks/networks)
│   ├── API/                            # OpenAPI-generated client + mappers
│   │   ├── Generated/                  # PhotonicClient/ (DO NOT EDIT)
│   │   └── Mappers/                    # DTO ↔ Domain transforms
│   ├── Auth/                           # OAuth2 flow, KeychainHelper, JWT
│   ├── Persistence/                    # SwiftData models (e.g., BackupSelection)
│   ├── Photos/                         # Photo library adapter
│   ├── Services/                       # Infrastructure services (BackupService, etc.)
│   └── Networking/                     # URLSession transport, middleware
│
├── Interface/                          # iOS UI (SwiftUI) — outer layer
│   ├── Screens/                        # Setup, Auth, Media, Backup, Settings
│   ├── Components/                     # Reusable SwiftUI components
│   ├── ViewModels/                     # Observable view models (talk to Services)
│   └── Previews/                       # Preview data/mocks
│
├── Config/
│   ├── openapi.yaml                    # API spec
│   └── openapi-generator.yml           # swift-openapi-generator config
│
├── Tests/
│   ├── Unit/                           # Domain & Application tests
│   ├── Integration/                    # API + adapters + persistence
│   ├── UI/ (PhotonicUITests)           # XCUITest
│   └── Fixtures/                       # Sample JSON, images, preview data
│
└── Tools/
    ├── Scripts/                        # Build/test scripts, codegen, lint
    └── Lint/                           # SwiftLint rules, SwiftFormat config
```

### Dependency Rules

- **Interface (UI)** depends on **Infrastructure services** and **Domain** protocols only.
- **Infrastructure services** depend on **Domain** and implement business logic.
- **Infrastructure adapters** implement **Domain** ports but do **not** leak into Domain.
- **App** composes everything via DI (build adapters, wire services, inject into UI).

---

## Essential Commands

### Build, Test, Clean

```bash
# Build
xcodebuild -scheme Photonic -configuration Debug build

# Unit + Integration tests
xcodebuild test -scheme Photonic -destination 'platform=iOS Simulator,name=iPhone 16'

# UI tests
xcodebuild test -scheme PhotonicUITests -destination 'platform=iOS Simulator,name=iPhone 16'

# Clean
xcodebuild clean -scheme Photonic
```

### OpenAPI Code Generation

```bash
# Regenerate API client from Config/openapi.yaml
swift package plugin generate-code-from-openapi
```

- Configure `openapi-generator.yml` to:

  - Generate decodable/encodable Swift types.
  - Use `URLSession` transport.
  - Namespace SDK into `PhotonicAPI`.
  - Avoid force-unwraps; prefer failable decoding.
- Generated code lives in `Infrastructure/API/Generated/` (ignored by
  lint/coverage).

---

## Architecture Details

### Domain (Entities & Value Objects)

- **Entities**: `MediaItem(id, checksum, createdAt, location?)`,
  `Album(id, name)`, `UserAccount(id, email)`.
- **Value Objects**: `AccessToken(value, expiresAt)`, `RefreshToken(value)`,
  `ServerURL(value)`, `MediaHash(value)`.
- **Rules**: Domain is pure; no imports from SwiftUI, Foundation allowed (non-IO
  only). No network, no Keychain.

### Infrastructure Services

- Examples:

  - `BackupService` (select albums → diff → enqueue uploads → report progress)
  - `AuthService` (ensure valid access token, rotate refresh token if needed)
  - `ServerDiscoveryService` (probe dev/staging/custom server URL)
- Services orchestrate between Domain entities and Infrastructure adapters.
- Use **Domain repositories** for `AuthRepository`, `MediaRepository`, `AlbumRepository`.
- Handle **transaction boundaries** and **retry policies** here.
- Services are in the Infrastructure layer and can depend on concrete implementations.

### Infrastructure (Adapters)

- **API Adapter**: Wrap generated client; map to Domain types; inject
  `AuthMiddleware` for bearer/refresh.
- **Auth Adapter**: OAuth2 via `ASWebAuthenticationSession` + PKCE; tokens in
  Keychain; JWT parsed via `Swift-JWT`.
- **Photos Adapter**: Access `PHPhotoLibrary`, request permissions, stream
  assets efficiently.
- **Persistence**: SwiftData models for UI state (e.g., `BackupSelection`), not
  for tokens.
- **Networking**: URLSession with timeouts, exponential backoff, connectivity
  awareness.

### Interface (SwiftUI)

- **ViewModels** are thin coordinators (Observable, @MainActor).
- Use **environment injection** for services.
- Keep views declarative, side-effect free (effects in ViewModels).

---

## Clean Code Guidelines

- Prefer **protocol-oriented** boundaries; don’t overuse classes where
  structs/actors suffice.
- **Actors** for shared mutable state (e.g., `AuthActor` guarding tokens).
- Descriptive names, single responsibility, small files.
- No force unwraps; use `XCTUnwrap` in tests.
- Centralize **error types** (`DomainError`, `NetworkError`, `AuthError`) and
  map them at boundaries.
- **Result builders** for UI composition, **async/await** for flows.
- Add **documentation comments** for all public types/methods crossing module
  boundaries.

---

## Security Hardening

- **OAuth2 + PKCE**, **ASWebAuthenticationSession**.
- **Keychain** for `AccessToken`/`RefreshToken`; never `UserDefaults`.
- Use **Secure Enclave** where appropriate (e.g., wrapping refresh token with
  Keychain access control).
- **Token Rotation**: On refresh, rotate and invalidate prior refresh token when
  backend supports it.
- **Least Privilege Scopes**; avoid overbroad claims.
- **ATS**: Enforce HTTPS; consider **certificate pinning** (store pins out of
  repo, rotateable).
- **Background uploads** via `BGProcessingTask` with transient credentials only.
- **PII-safe logging**: Never log tokens, emails, or image identifiers.
- **Jailbreak/Debug hints** (best-effort): degrade features if high risk is
  detected; never rely solely on this.
- **Threat modeling checklist** in PR template (auth, transport, storage,
  privacy).
- **Crash safety**: scrub breadcrumbs and redact data before sending to crash
  reporters.

---

## OpenAPI Integration

- Source of truth: `Config/openapi.yaml`.

- After modifying:

  1. Update `openapi.yaml`.
  2. Run codegen (above).
  3. Update API mappers in `Infrastructure/API/Mappers/`.
  4. Verify **contract tests** (Integration) against dev/staging.

- **Mappers**: Keep strict, explicit mapping functions
  (`API.MediaDTO → Domain.MediaItem`).

- **Resilience**: Treat unknown fields as non-fatal; surface **server error
  codes** via typed errors.

---

## Testing Strategy

### Unit Tests (Tests/Unit)

- Target: **Domain** + **Infrastructure Services**.
- Use fakes/mocks for repositories: `FakeAuthRepository`, `InMemoryMediaRepository`, etc.
- Test domain invariants (e.g., `MediaHash` uniqueness), services (e.g.,
  `BackupService` happy/sad paths).
- **Coverage targets**: Domain 90%+, Services 80%+.

### Integration Tests (Tests/Integration)

- Test adapters with real frameworks where feasible:

  - API adapter against a **local mock server** (e.g., lightweight Vapor stub or
    URLProtocol stubs).
  - Persistence adapter with ephemeral stores.
  - Auth adapter with simulated OAuth callback URLs.
- Verify mapping correctness and error translation end-to-end.

### UI Tests (Tests/UI – XCUITest)

- Launch arguments to use **mock ports** and **in-memory stores**.
- Network is stubbed; deterministic flows.
- Include **accessibility checks**, screen **snapshots**, and **critical path**
  (first run → setup → login → select albums → start backup → progress →
  completion).
- Prefer identifiers: `accessibilityIdentifier("backupStartButton")`.

### Test Commands

```bash
# Unit
xcodebuild test -scheme Photonic -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:PhotonicTests

# Integration
xcodebuild test -scheme Photonic -destination 'platform=iOS Simulator,name=iPhone 16' -only-testing:PhotonicIntegrationTests

# UI
xcodebuild test -scheme PhotonicUITests -destination 'platform=iOS Simulator,name=iPhone 16'
```

---

## SwiftUI Previews (First-class)

- All views must include **standalone Previews** with **mock ViewModels** and
  **dependency injection**.
- Use `Interface/Previews/PreviewDependencies.swift` for shared fake services
  and sample data.
- Previews shouldn’t hit Keychain/network/Photos; always feed static data.
- Provide multiple **preview states**: empty, loading, success with varied data,
  error, dark mode, dynamic type sizes, RTL.
- Example:

```swift
struct MediaGridView_Previews: PreviewProvider {
    static var previews: some View {
        MediaGridView(
            viewModel: .init(
                listMedia: PreviewDeps.listMediaSuccess,
                selection: .mockAlbums
            )
        )
        .environment(\.locale, .init(identifier: "de_DE"))
        .environment(\.colorScheme, .dark)
        .environment(\.sizeCategory, .accessibilityExtraExtraExtraLarge)
    }
}
```

> **Inject**: Allowed in dev only. Guard with `#if DEBUG` and keep production
> builds free from Inject.

---

## Concurrency & Middleware

- **AuthActor**: sole writer to token state; exposes async APIs.
- **AuthMiddleware** (URLProtocol or per-request adapter): injects bearer; on
  401, refresh via `AuthActor` with single-flight semantics; retries idempotent
  requests only.
- **Backoff**: exponential, jittered; surface cancellation to UI.

---

## DX: Linting, Formatting, CI

- **SwiftLint**: Enable rules for force\_unwrap, todo, large\_tuple,
  cyclomatic\_complexity.
- **SwiftFormat**: Enforce style in CI.
- **CI pipeline**:

  1. Cache SPM
  2. Lint & format check
  3. Build
  4. Generate OpenAPI client (verify no diff)
  5. Run Unit → Integration → UI tests
  6. Artifact: screenshots, code coverage, preview catalog (optional)
- **PR Template**: checklist for architecture, tests added, previews added,
  security review done.

---

## Setup & Environments

- **Servers**:

  - Development: `http://localhost:8080`
  - Staging: `https://photonic.mvissing.de`

- **Configuration**:

  - Use a `ConfigService` (Application) for base URL selection.
  - Don’t hardcode secrets; use build settings or configuration files excluded
    from VCS.
  - Store user server URL selection (if custom) as a **Value Object** validated
    at the boundary.

---

## Migration Path (from current layout)

1. **Introduce ports** in Domain/Application for existing concrete services
   (Auth, API, Photos, Uploads).
2. **Wrap generated API** with an adapter implementing the ports.
3. **Move SwiftUI ViewModels** to **Interface/ViewModels**; convert to depend on
   **Services** only.
4. **Extract** Keychain and SwiftData into Infrastructure adapters; replace
   direct calls in UI.
5. **Add previews** with PreviewDeps and mocks for each screen.
6. **Expand tests**: begin with Domain invariants → Service flows → Adapters →
   UI happy paths.
7. **Enforce lint/format** and CI gates.

---

## Example Service Architecture (sketch)

```swift
// Domain/Repositories/AuthRepository.swift (protocol)
public protocol AuthRepository {
    func currentTokens() async throws -> (access: AccessToken, refresh: RefreshToken)
    func refreshTokensIfNeeded() async throws -> AccessToken
    func signInInteractive() async throws
    func signOut() async throws
}

// Domain/Repositories/MediaRepository.swift (protocol)
public protocol MediaRepository {
    func listAlbums() async throws -> [Album]
    func listMedia(in albums: [Album]) async throws -> AsyncThrowingStream<MediaItem, Error>
    func upload(_ item: MediaItem) async throws -> UploadResult
}

// Infrastructure/Services/BackupService.swift
public protocol BackupServiceProtocol {
    func startBackup(for selections: [BackupAlbumSelectionEntity]) async throws -> AsyncThrowingStream<BackupProgress, Error>
}
```

Infrastructure adapters implement Domain repository protocols, and Infrastructure services orchestrate business logic using these repositories.

---

## Error Handling & Observability

- **Typed errors**, mapped at boundaries; UI gets user-readable messages.
- **Metrics**: track upload throughput, failures, retries (privacy-safe).
- **Logging**: OSLog categories (Auth, Network, Upload); redact sensitive
  fields.

---

## Hot Reload Development

- Inject is available during development for SwiftUI hot reload.
- Wrap views with `@ObserveInjection` **under `#if DEBUG` only**.
- Never compile Inject into release builds.

---

## Quick Reference: Scripts

Place in `Tools/Scripts/` and wire in CI:

```bash
./Tools/Scripts/lint.sh
./Tools/Scripts/format.sh
./Tools/Scripts/generate_openapi.sh
./Tools/Scripts/test_all.sh
```

---

## Dependencies

Via Swift Package Manager:

- `swift-openapi-generator` (API client generation)
- `OAuth2` (OAuth2/OIDC)
- `Swift-JWT` (JWT parsing)
- `Inject` (dev hot reload — DEBUG only)

Full list in `Package.resolved`.

---

### Final Notes for Contributors (and Claude)

- Favor **composition over inheritance**.
- Keep **UI dumb, use cases smart, domain pure**.
- Every new screen ships with **previews + tests**.
- Any API change → **update spec, regenerate client, update mappers, run
  contract tests**.
- Treat **security** as a feature: reviewed in every PR.

