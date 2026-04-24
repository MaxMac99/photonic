//
//  ServerConfiguration.swift
//  Photonic
//
//  Domain Value Object
//

import Foundation

struct ServerConfiguration: Equatable, Codable {
    let serverUrl: ServerURL
    let clientId: String
    let tokenUrl: URL
    let authorizationUrl: URL

    init?(serverUrl: URL, clientId: String, tokenUrl: URL, authorizationUrl: URL) {
        guard let serverURL = ServerURL(url: serverUrl) else { return nil }
        self.serverUrl = serverURL
        self.clientId = clientId
        self.tokenUrl = tokenUrl
        self.authorizationUrl = authorizationUrl
    }

    /// For Codable compatibility
    private enum CodingKeys: String, CodingKey {
        case serverUrl, clientId, tokenUrl, authorizationUrl
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let url = try container.decode(URL.self, forKey: .serverUrl)
        guard let serverURL = ServerURL(url: url) else {
            throw DecodingError.dataCorruptedError(forKey: .serverUrl, in: container, debugDescription: "Invalid server URL")
        }
        serverUrl = serverURL
        clientId = try container.decode(String.self, forKey: .clientId)
        tokenUrl = try container.decode(URL.self, forKey: .tokenUrl)
        authorizationUrl = try container.decode(URL.self, forKey: .authorizationUrl)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(serverUrl.value, forKey: .serverUrl)
        try container.encode(clientId, forKey: .clientId)
        try container.encode(tokenUrl, forKey: .tokenUrl)
        try container.encode(authorizationUrl, forKey: .authorizationUrl)
    }
}
