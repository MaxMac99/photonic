//
//  UserRepository.swift
//  Photonic
//
//  Domain Repository Protocol
//

import Foundation

protocol UserRepository {
    func getUserStats() async throws -> UserStats
}

struct UserStats {
    let albums: UInt64
    let media: UInt64
    let quota: UInt64
    let quotaUsed: UInt64
}
