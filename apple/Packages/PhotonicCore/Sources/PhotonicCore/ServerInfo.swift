import Foundation

/// The discovery payload of a Photonic server (`system_info`): the server's
/// version and the OAuth endpoints a client needs to authenticate.
/// Mapped and validated at the API boundary; consumed by setup, auth, and
/// settings.
public struct ServerInfo: Hashable, Sendable, Codable {
    public let version: String
    public let clientID: String
    public let tokenURL: ServerURL
    public let authorizeURL: ServerURL

    public init(version: String, clientID: String, tokenURL: ServerURL, authorizeURL: ServerURL) {
        self.version = version
        self.clientID = clientID
        self.tokenURL = tokenURL
        self.authorizeURL = authorizeURL
    }
}
