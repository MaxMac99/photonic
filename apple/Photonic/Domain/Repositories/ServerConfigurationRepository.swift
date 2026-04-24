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
    let clientId: String
    let authorizeUrl: URL
    let tokenUrl: URL
    let serverVersion: String?
}
