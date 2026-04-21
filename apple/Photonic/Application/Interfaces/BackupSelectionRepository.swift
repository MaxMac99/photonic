//
//  BackupSelectionRepository.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import Foundation

public protocol BackupSelectionRepository {
    func fetchSelections() async throws -> [BackupAlbumSelectionEntity]
    func saveSelection(_ selection: BackupAlbumSelectionEntity) async throws
    func deleteSelection(withId id: String) async throws
    func findSelection(byAlbumId albumId: String) async throws -> BackupAlbumSelectionEntity?
    func clearAllSelections() async throws
}

public struct BackupAlbumSelectionEntity: Equatable, Identifiable {
    public let id: String
    public let albumIdentifier: String
    public let albumName: String
    public let selectionType: AlbumSelectionType
    public let createdAt: Date
    
    public init(
        id: String = UUID().uuidString,
        albumIdentifier: String,
        albumName: String,
        selectionType: AlbumSelectionType,
        createdAt: Date = Date()
    ) {
        self.id = id
        self.albumIdentifier = albumIdentifier
        self.albumName = albumName
        self.selectionType = selectionType
        self.createdAt = createdAt
    }
}

public enum AlbumSelectionType: Equatable {
    case included
    case excluded
}