//
//  RefreshToken.swift
//  Photonic
//
//  Domain Value Object
//

import Foundation

struct RefreshToken: Equatable {
    let value: String
    let issuedAt: Date

    init(value: String, issuedAt: Date = Date()) {
        self.value = value
        self.issuedAt = issuedAt
    }
}
