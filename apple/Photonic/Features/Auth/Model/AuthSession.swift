import Foundation
import PhotonicCore

/// An authenticated user session: which server, its OAuth endpoints, and the
/// current tokens. Persisted as one JSON blob in the Keychain.
struct AuthSession: Equatable, Sendable, Codable {
    let serverURL: ServerURL
    let serverInfo: ServerInfo
    let accessToken: AccessToken
    let refreshToken: RefreshToken
}

enum AuthError: Error, Sendable {
    case serverNotConfigured
    case invalidAuthorizeURL
    case invalidCallback
    case tokenExchangeFailed
    case invalidTokenResponse
}
