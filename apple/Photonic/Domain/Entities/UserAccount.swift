//
//  UserAccount.swift
//  Photonic
//
//  Domain Entity
//

import Foundation

struct UserAccount: Equatable, Hashable {
    let id: String
    let email: String
    let name: String?
    let givenName: String?
    let familyName: String?
    let nickname: String?
    let preferredUsername: String?
    let profileUrl: String?
    let pictureUrl: String?
    let emailVerified: Bool
    let quota: Quota?
    let createdAt: Date
    
    struct Quota: Equatable, Hashable {
        let totalBytes: Int64
        let usedBytes: Int64
        
        var availableBytes: Int64 {
            totalBytes - usedBytes
        }
        
        var usagePercentage: Double {
            guard totalBytes > 0 else { return 0 }
            return Double(usedBytes) / Double(totalBytes) * 100
        }
    }
}

extension UserAccount {
    var displayName: String {
        if let name = name, !name.isEmpty {
            return name
        }
        if let preferredUsername = preferredUsername, !preferredUsername.isEmpty {
            return preferredUsername
        }
        if let nickname = nickname, !nickname.isEmpty {
            return nickname
        }
        return email
    }
}