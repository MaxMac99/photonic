//
//  ServerConfigurationRepositoryImpl.swift
//  Photonic
//
//  Real implementation of ServerConfigurationRepository
//

import Foundation
import OpenAPIURLSession

final class ServerConfigurationRepositoryImpl: ServerConfigurationRepository {
    private let userDefaults: UserDefaults
    private let configurationKey = "de.photonic.serverConfiguration"

    init(userDefaults: UserDefaults = .standard) {
        self.userDefaults = userDefaults
    }

    func getCurrentConfiguration() async throws -> ServerConfiguration? {
        guard let data = userDefaults.data(forKey: configurationKey) else {
            return nil
        }

        do {
            let decoder = JSONDecoder()
            return try decoder.decode(ServerConfiguration.self, from: data)
        } catch {
            throw DomainError.decoding("Failed to decode server configuration: \(error)")
        }
    }

    func saveConfiguration(_ configuration: ServerConfiguration) async throws {
        do {
            let encoder = JSONEncoder()
            let data = try encoder.encode(configuration)
            userDefaults.set(data, forKey: configurationKey)
        } catch {
            throw DomainError.encoding("Failed to encode server configuration: \(error)")
        }
    }

    func deleteConfiguration() async throws {
        userDefaults.removeObject(forKey: configurationKey)
    }

    func discoverServerInfo(url: URL) async throws -> ServerDiscoveryInfo {
        let client = Client(
            serverURL: url,
            transport: URLSessionTransport()
        )

        do {
            let response = try await client.system_info()
            let info = try response.ok.body.json

            // Map the API response to our domain model. The OAuth fields are
            // null when the server has OIDC disabled; discovery still succeeds
            // so the app can connect without authentication.
            return ServerDiscoveryInfo(
                clientId: info.client_id,
                authorizeUrl: info.authorize_url.flatMap(URL.init(string:)),
                tokenUrl: info.token_url.flatMap(URL.init(string:)),
                serverVersion: info.version
            )
        } catch {
            throw DomainError.networkError("Failed to discover server info: \(error)")
        }
    }
}
