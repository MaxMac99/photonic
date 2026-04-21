//
//  ServerInfo.swift
//  Photonic
//
//  Created by Max Vissing on 10.01.25.
//

import Foundation

struct ServerInfo: Codable {
    let serverUrl: URL
    let clientId: String
    let tokenUrl: URL
    let authorizationUrl: URL
}
