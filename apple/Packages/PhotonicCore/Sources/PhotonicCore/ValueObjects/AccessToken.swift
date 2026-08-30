import Foundation

/// A bearer access token with its expiry. Validated at the boundary where
/// the auth response is parsed; never persisted outside the Keychain adapter.
public struct AccessToken: Hashable, Sendable, Codable {
    public let value: String
    public let expiresAt: Date

    public init?(value: String, expiresAt: Date) {
        guard !value.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return nil }
        self.value = value
        self.expiresAt = expiresAt
    }

    /// Pure time check — the caller supplies the clock.
    public func isExpired(at date: Date = Date()) -> Bool {
        date >= expiresAt
    }
}
