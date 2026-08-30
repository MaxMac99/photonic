import Foundation

/// A refresh token. Validated at the boundary where the auth response is
/// parsed; never persisted outside the Keychain adapter.
public struct RefreshToken: Hashable, Sendable, Codable {
    public let value: String

    public init?(value: String) {
        guard !value.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return nil }
        self.value = value
    }
}
