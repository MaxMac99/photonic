# Photonic iOS App

Photonic is an iOS application for backing up photos from your device to a Photonic server. The app is built with SwiftUI and follows Hexagonal Architecture principles with Domain-Driven Design (DDD).

## Architecture Overview

The application follows **Hexagonal Architecture** (Ports and Adapters) combined with **Domain-Driven Design** principles to maintain a clean, testable, and maintainable codebase.

### Directory Structure

```
Photonic/
├── App/                                # iOS app target (composition root)
│   ├── PhotonicApp.swift               # Entry point, DI wiring, Scene
│   └── CompositionRoot.swift           # Build adapters & inject use cases
│
├── Domain/                             # DDD Core (pure Swift)
│   ├── Entities/                       # MediaItem, Album, UserAccount
│   ├── ValueObjects/                   # Token, ServerURL, MediaHash
│   ├── Services/                       # Domain services (pure logic)
│   ├── Repositories/                   # Protocol definitions (ports)
│   └── Errors/                         # DomainError, AuthError
│
├── Application/                        # Use cases (ports & orchestrations)
│   ├── UseCases/                       # BackupMedia, RefreshAuth, DiscoverServer
│   ├── DTOs/                           # App-layer models if needed
│   └── Interfaces/                     # Application ports (protocols)
│
├── Infrastructure/                     # Adapters (to frameworks/networks)
│   ├── API/                            # OpenAPI-generated client + mappers
│   │   ├── Generated/                  # PhotonicClient/ (DO NOT EDIT)
│   │   └── Mappers/                    # DTO ↔ Domain transforms
│   ├── Auth/                           # OAuth2 flow, KeychainHelper, JWT
│   ├── Persistence/                    # SwiftData models
│   ├── Photos/                         # Photo library adapter
│   └── Networking/                     # URLSession transport, middleware
│
├── Interface/                          # iOS UI (SwiftUI) — outer layer
│   ├── Screens/                        # Setup, Auth, Media, Backup, Settings
│   ├── Components/                     # Reusable SwiftUI components
│   ├── ViewModels/                     # Observable view models
│   └── Previews/                       # Preview data/mocks
│
├── Config/
│   ├── openapi.yaml                    # API specification
│   └── openapi-generator.yml           # swift-openapi-generator config
│
└── Tests/
    ├── Unit/                           # Domain & Application tests
    ├── Integration/                    # API + adapters + persistence
    └── UI/                             # XCUITest
```

### Layer Dependencies

The architecture enforces strict dependency rules:

- **Domain Layer**: Pure business logic, no external dependencies
- **Application Layer**: Depends only on Domain
- **Infrastructure Layer**: Implements Domain/Application ports
- **Interface Layer**: Depends on Application (use cases)
- **App Layer**: Wires everything together (Composition Root)

## Key Components

### Domain Layer
- **Entities**: Core business objects (`MediaItem`, `Album`, `UserAccount`)
- **Value Objects**: Immutable domain concepts (`AccessToken`, `RefreshToken`, `ServerURL`)
- **Repository Protocols**: Interfaces for data access
- **Domain Services**: Pure business logic operations

### Application Layer
- **Use Cases**: Application-specific business rules
  - `DiscoverServerUseCase`: Server discovery and configuration
  - `BackupMediaUseCase`: Photo backup orchestration
  - `RefreshAuthUseCase`: Token management
- **DTOs**: Data transfer objects for boundary crossing

### Infrastructure Layer
- **Auth**: OAuth2/OIDC implementation with PKCE
  - `AuthManager`: OAuth flow management
  - `KeychainHelper`: Secure token storage
  - `AuthRepositoryImpl`: Auth repository implementation
- **API**: Generated OpenAPI client and mappers
- **Persistence**: SwiftData for local state
- **Photos**: iOS Photos framework integration

### Interface Layer
- **Screens**: Main UI screens
- **ViewModels**: Presentation logic and state management
- **Components**: Reusable UI components
- **Previews**: SwiftUI preview providers with mock data

## Authentication

The app uses OAuth2 with PKCE for secure authentication:

1. Server discovery via `.well-known/openid-configuration`
2. OAuth2 authorization flow with PKCE challenge
3. Secure token storage in iOS Keychain
4. Automatic token refresh

### Keychain Storage

Sensitive data like OAuth tokens are stored securely using the `KeychainHelper` class:

```swift
// Store token
let tokenData = token.data(using: .utf8)!
try KeychainHelper.upsertData(
    data: tokenData,
    forService: "com.photonic.oauth",
    account: "access_token"
)

// Retrieve token
let data = try KeychainHelper.getData(
    forService: "com.photonic.oauth",
    account: "access_token"
)
```

## Building the Project

### Requirements
- Xcode 15.0+
- iOS 17.0+
- Swift 5.9+

### Build Commands

```bash
# Build for Debug
xcodebuild -scheme Photonic -configuration Debug build

# Run Unit Tests
xcodebuild test -scheme Photonic -destination 'platform=iOS Simulator,name=iPhone 16'

# Run UI Tests
xcodebuild test -scheme PhotonicUITests -destination 'platform=iOS Simulator,name=iPhone 16'

# Clean
xcodebuild clean -scheme Photonic
```

### OpenAPI Code Generation

The API client is generated from the OpenAPI specification:

```bash
# Regenerate API client
swift package plugin generate-code-from-openapi
```

Configuration is in `Config/openapi-generator.yml`.

## Development

### SwiftUI Previews

All views include preview providers with mock data for development:

```swift
struct MediaGridView_Previews: PreviewProvider {
    static var previews: some View {
        MediaGridView(
            viewModel: .init(
                listMedia: PreviewDeps.listMediaSuccess,
                selection: .mockAlbums
            )
        )
    }
}
```

### Hot Reload

During development, hot reload is available using Inject (DEBUG builds only).

## Testing Strategy

### Unit Tests
- Domain logic validation
- Use case business rules
- Coverage targets: Domain 90%+, Application 80%+

### Integration Tests
- API adapter testing with mock servers
- Persistence adapter verification
- Auth flow validation

### UI Tests
- Critical user paths
- Accessibility verification
- Visual regression testing

## Security

- **OAuth2 + PKCE**: Secure authentication flow
- **Keychain Storage**: Encrypted credential storage
- **Token Rotation**: Automatic refresh token rotation
- **HTTPS Only**: Enforced except for localhost development
- **No Logging of PII**: Tokens and user data never logged

## Contributing

Please refer to [CLAUDE.md](CLAUDE.md) for detailed development guidelines and architecture decisions.

## License

[License information here]