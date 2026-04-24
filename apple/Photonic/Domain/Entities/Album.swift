//
//  Album.swift
//  Photonic
//
//  Domain Entity
//

import Foundation

public struct Album: Equatable, Hashable {
    public let id: String
    public let name: String
    public let createdAt: Date
    public let updatedAt: Date
    public let mediaCount: Int
    public let coverMediaId: String?
    public let isShared: Bool
    public let ownerUserId: String

    public init(
        id: String,
        name: String,
        createdAt: Date = Date(),
        updatedAt: Date = Date(),
        mediaCount: Int = 0,
        coverMediaId: String? = nil,
        isShared: Bool = false,
        ownerUserId: String = ""
    ) {
        self.id = id
        self.name = name
        self.createdAt = createdAt
        self.updatedAt = updatedAt
        self.mediaCount = mediaCount
        self.coverMediaId = coverMediaId
        self.isShared = isShared
        self.ownerUserId = ownerUserId
    }

    /// Convenience initializer for local albums
    public init(id: String, name: String, assetCount: Int) {
        self.init(
            id: id,
            name: name,
            mediaCount: assetCount
        )
    }
}

extension Album {
    var isEmpty: Bool {
        mediaCount == 0
    }
}
