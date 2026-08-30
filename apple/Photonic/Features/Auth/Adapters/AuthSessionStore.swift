import Foundation
import PhotonicCore

/// Minimal Keychain CRUD for the auth session blob. Keychain usage is
/// confined to this adapter (R3); tokens never touch UserDefaults.
enum AuthSessionStore {
    private static let service = "de.maxvissing.Photonic"
    private static let account = "authSession"

    static func load() -> AuthSession? {
        var query = baseQuery
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess, let data = result as? Data else { return nil }
        return try? JSONDecoder().decode(AuthSession.self, from: data)
    }

    static func save(_ session: AuthSession) throws {
        guard let data = try? JSONEncoder().encode(session) else {
            throw AuthError.invalidTokenResponse
        }
        delete()

        var query = baseQuery
        query[kSecValueData as String] = data
        query[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly

        let status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else { throw AuthError.tokenExchangeFailed }
    }

    static func delete() {
        SecItemDelete(baseQuery as CFDictionary)
    }

    private static var baseQuery: [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
    }
}
