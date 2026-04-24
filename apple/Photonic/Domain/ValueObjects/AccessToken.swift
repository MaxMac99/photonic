//
//  AccessToken.swift
//  Photonic
//
//  Domain Value Object
//

import Foundation

struct AccessToken: Equatable {
    let value: String
    let expiresAt: Date
    let scopes: Set<String>

    init(value: String, expiresAt: Date, scopes: Set<String> = []) {
        self.value = value
        self.expiresAt = expiresAt
        self.scopes = scopes
    }

    var isExpired: Bool {
        Date() >= expiresAt
    }

    var isExpiringSoon: Bool {
        let fiveMinutesFromNow = Date().addingTimeInterval(5 * 60)
        return fiveMinutesFromNow >= expiresAt
    }

    func hasScope(_ scope: String) -> Bool {
        scopes.contains(scope)
    }
}
