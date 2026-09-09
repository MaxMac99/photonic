import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicCore

/// Typed interface to the authentication lifecycle: restoring a stored
/// session, running the OAuth2+PKCE sign-in flow, and signing out.
@DependencyClient
struct AuthClient: Sendable {
    var restoreSession: @Sendable () async -> AuthSession? = { nil }
    var signIn: @Sendable (ServerURL, ServerInfo) async throws -> AuthSession
    var signOut: @Sendable () async -> Void = {}
}

extension DependencyValues {
    var authClient: AuthClient {
        get { self[AuthClient.self] }
        set { self[AuthClient.self] = newValue }
    }
}
