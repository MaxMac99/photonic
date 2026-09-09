import Foundation

/// The user's configured Photonic server. Persisted by the Settings
/// feature's adapter; read app-wide (auth, API, backup).
public struct ServerConfiguration: Hashable, Sendable, Codable {
    public let serverURL: ServerURL

    public init(serverURL: ServerURL) {
        self.serverURL = serverURL
    }
}
