//
//  ServerInfo.swift
//  Photonic
//
//  Created by Max Vissing on 10.01.25.
//

import Foundation

struct ServerInfo: Codable {
    let serverUrl: URL
    /// `nil` when the server has OIDC disabled
    let clientId: String?
    /// `nil` when the server has OIDC disabled
    let tokenUrl: URL?
    /// `nil` when the server has OIDC disabled
    let authorizationUrl: URL?
}
