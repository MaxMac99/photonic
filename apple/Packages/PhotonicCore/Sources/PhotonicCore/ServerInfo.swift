import Foundation

/// The discovery payload of a Photonic server (`system_info`): the server's
/// version and — when OIDC is enabled — the OAuth endpoints a client needs
/// to authenticate. Mapped and validated at the API boundary; consumed by
/// setup, auth, and settings.
public struct ServerInfo: Hashable, Sendable, Codable {
    public let version: String
    /// `nil` when the server has OIDC disabled.
    public let clientID: String?
    /// `nil` when the server has OIDC disabled.
    public let tokenURL: ServerURL?
    /// `nil` when the server has OIDC disabled.
    public let authorizeURL: ServerURL?

    public init(
        version: String,
        clientID: String?,
        tokenURL: ServerURL?,
        authorizeURL: ServerURL?
    ) {
        self.version = version
        self.clientID = clientID
        self.tokenURL = tokenURL
        self.authorizeURL = authorizeURL
    }

    /// Whether interactive OAuth sign-in is available on this server.
    public var isOIDCEnabled: Bool {
        clientID != nil && tokenURL != nil && authorizeURL != nil
    }
}
