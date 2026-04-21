//
//  KeychainHelper.swift
//  Photonic
//
//  Created by Max Vissing on 02.05.24.
//

import Foundation
import Security

/// Custom errors that can occur during Keychain operations.
///
/// These errors provide more context than raw OSStatus codes and help
/// identify specific failure scenarios when working with the Keychain.
enum KeychainError: Error {
    /// The requested item was not found in the Keychain.
    case itemNotFound
    /// An item with the same service and account already exists.
    case duplicateItem
    /// An unexpected error occurred with the given status code.
    case unexpectedStatus(OSStatus)
}

/// A utility class for securely storing and retrieving sensitive data in the iOS Keychain.
///
/// `KeychainHelper` provides a type-safe interface to the Security framework's Keychain Services,
/// specifically for storing generic password items. It handles common operations like adding,
/// retrieving, updating, and deleting sensitive data such as OAuth tokens.
///
/// ## Security Considerations
/// - All data is stored using `kSecClassGenericPassword` class
/// - Data is encrypted by the system and protected by the device lock
/// - Access is limited to the app that created the items (default access group)
/// - Items persist across app installations unless explicitly deleted
///
/// ## Usage Example
/// ```swift
/// // Storing a token
/// let tokenData = token.data(using: .utf8)!
/// try KeychainHelper.addData(data: tokenData, forService: "com.app.oauth", account: "access_token")
///
/// // Retrieving a token
/// let data = try KeychainHelper.getData(forService: "com.app.oauth", account: "access_token")
/// let token = String(data: data, encoding: .utf8)
/// ```
///
/// - Important: This class uses static methods and does not maintain state.
///              All operations are atomic and thread-safe at the Security framework level.
class KeychainHelper {

    /// Adds new data to the Keychain for the specified service and account.
    ///
    /// This method creates a new Keychain item with the provided data. It will fail
    /// if an item with the same service and account combination already exists.
    ///
    /// - Parameters:
    ///   - data: The data to store securely in the Keychain.
    ///   - service: A unique identifier for the service (e.g., "com.app.oauth").
    ///   - account: The account or key name for this item (e.g., "access_token").
    ///
    /// - Throws:
    ///   - `KeychainError.duplicateItem` if an item already exists with the same service/account
    ///   - `KeychainError.unexpectedStatus` for other Keychain errors
    ///
    /// - Note: Use `upsertData` if you want to add or update in a single operation.
    static func addData(data: Data, forService service: String, account: String) throws {
        let query =
            [
                kSecClass as String: kSecClassGenericPassword,
                kSecAttrService as String: service,
                kSecAttrAccount as String: account,
                kSecValueData as String: data,
            ] as [String: Any]

        let status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else {
            if status == errSecDuplicateItem {
                throw KeychainError.duplicateItem
            }
            throw KeychainError.unexpectedStatus(status)
        }
    }

    /// Retrieves data from the Keychain for the specified service and account.
    ///
    /// This method queries the Keychain for an item matching the service and account
    /// combination and returns its data.
    ///
    /// - Parameters:
    ///   - service: The service identifier used when the item was stored.
    ///   - account: The account identifier used when the item was stored.
    ///
    /// - Returns: The data stored in the Keychain for the given service and account.
    ///
    /// - Throws:
    ///   - `KeychainError.itemNotFound` if no item exists with the given service/account
    ///   - `KeychainError.unexpectedStatus` for other Keychain errors
    ///
    /// - Note: The returned data is exactly as stored; any encoding/decoding is the caller's responsibility.
    static func getData(forService service: String, account: String) throws -> Data {
        let query =
            [
                kSecClass as String: kSecClassGenericPassword,
                kSecAttrService as String: service,
                kSecAttrAccount as String: account,
                kSecMatchLimit as String: kSecMatchLimitOne,
                kSecReturnData as String: true,
            ] as [String: Any]

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        guard status == errSecSuccess else {
            if status == errSecItemNotFound {
                throw KeychainError.itemNotFound
            }
            throw KeychainError.unexpectedStatus(status)
        }

        guard let data = result as? Data else {
            throw KeychainError.unexpectedStatus(status)
        }

        return data
    }

    /// Updates existing data in the Keychain for the specified service and account.
    ///
    /// This method modifies the data of an existing Keychain item. The item must
    /// already exist for this operation to succeed.
    ///
    /// - Parameters:
    ///   - data: The new data to store, replacing the existing data.
    ///   - service: The service identifier of the item to update.
    ///   - account: The account identifier of the item to update.
    ///
    /// - Throws:
    ///   - `KeychainError.itemNotFound` if no item exists with the given service/account
    ///   - `KeychainError.unexpectedStatus` for other Keychain errors
    ///
    /// - Note: Use `upsertData` if the item might not exist and you want to create it.
    static func updateData(data: Data, forService service: String, account: String) throws {
        let query =
            [
                kSecClass as String: kSecClassGenericPassword,
                kSecAttrService as String: service,
                kSecAttrAccount as String: account,
            ] as [String: Any]
        let attributes =
            [
                kSecValueData as String: data
            ] as [String: Any]

        let status = SecItemUpdate(query as CFDictionary, attributes as CFDictionary)
        guard status == errSecSuccess else {
            if status == errSecItemNotFound {
                throw KeychainError.itemNotFound
            }
            throw KeychainError.unexpectedStatus(status)
        }
    }

    /// Inserts or updates data in the Keychain for the specified service and account.
    ///
    /// This method provides "upsert" functionality - it will update an existing item
    /// if found, or create a new item if not found. This is useful when you don't
    /// know whether an item already exists.
    ///
    /// - Parameters:
    ///   - data: The data to store or update in the Keychain.
    ///   - service: The service identifier for the item.
    ///   - account: The account identifier for the item.
    ///
    /// - Throws:
    ///   - `KeychainError.unexpectedStatus` for Keychain errors other than item not found
    ///
    /// - Note: This method internally handles the `itemNotFound` error when attempting
    ///         to update, automatically falling back to adding a new item.
    static func upsertData(data: Data, forService service: String, account: String) throws {
        do {
            _ = try getData(forService: service, account: account)
            try updateData(data: data, forService: service, account: account)
        } catch KeychainError.itemNotFound {
            try addData(data: data, forService: service, account: account)
        }
    }

    /// Deletes data from the Keychain for the specified service and account.
    ///
    /// This method removes a Keychain item matching the service and account combination.
    /// It will succeed even if the item doesn't exist (idempotent operation).
    ///
    /// - Parameters:
    ///   - service: The service identifier of the item to delete.
    ///   - account: The account identifier of the item to delete.
    ///
    /// - Throws:
    ///   - `KeychainError.unexpectedStatus` for Keychain errors other than item not found
    ///
    /// - Important: This operation is permanent. Deleted items cannot be recovered.
    ///
    /// - Note: The method treats "item not found" as a success case, making it safe
    ///         to call multiple times or when uncertain if an item exists.
    static func deleteData(forService service: String, account: String) throws {
        let query =
            [
                kSecClass as String: kSecClassGenericPassword,
                kSecAttrService as String: service,
                kSecAttrAccount as String: account,
            ] as [String: Any]

        let status = SecItemDelete(query as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw KeychainError.unexpectedStatus(status)
        }
    }
}
