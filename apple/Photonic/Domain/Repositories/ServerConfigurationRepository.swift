//
//  ServerConfigurationRepository.swift
//  Photonic
//
//  Domain Repository Protocol
//

import Foundation

protocol ServerConfigurationRepository {
    func getCurrentConfiguration() async throws -> ServerConfiguration?
    func saveConfiguration(_ configuration: ServerConfiguration) async throws
    func deleteConfiguration() async throws
    func discoverServerInfo(url: URL) async throws -> ServerDiscoveryInfo
}

struct ServerDiscoveryInfo: Equatable {
    /// `nil` when the server has OIDC disabled
    let clientId: String?
    /// `nil` when the server has OIDC disabled
    let authorizeUrl: URL?
    /// `nil` when the server has OIDC disabled
    let tokenUrl: URL?
    let serverVersion: String?

    /// Whether the server offers OAuth login (all OAuth settings present)
    var hasOAuth: Bool {
        clientId != nil && authorizeUrl != nil && tokenUrl != nil
    }
}
