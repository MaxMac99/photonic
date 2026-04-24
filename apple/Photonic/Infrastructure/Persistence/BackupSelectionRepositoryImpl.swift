//
//  BackupSelectionRepositoryImpl.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import Foundation
import OSLog
import SwiftData

public final class BackupSelectionRepositoryImpl: BackupSelectionRepository {
    private static let logger = Logger(
        subsystem: Bundle.main.bundleIdentifier!,
        category: String(describing: BackupSelectionRepositoryImpl.self)
    )

    private let modelContext: ModelContext

    public init(modelContext: ModelContext) {
        self.modelContext = modelContext
    }

    public func fetchSelections() async throws -> [BackupAlbumSelectionEntity] {
        let descriptor = FetchDescriptor<BackupAlbumSelection>(
            sortBy: [SortDescriptor(\.creationDate)]
        )

        let selections = try modelContext.fetch(descriptor)

        return selections.map { selection in
            BackupAlbumSelectionEntity(
                id: selection.localIdentifier,
                albumIdentifier: selection.localIdentifier,
                albumName: selection.name,
                selectionType: selection.selectionType == .included ? .included : .excluded,
                createdAt: selection.creationDate
            )
        }
    }

    public func saveSelection(_ selection: BackupAlbumSelectionEntity) async throws {
        // Check if already exists
        let identifier = selection.albumIdentifier
        let descriptor = FetchDescriptor<BackupAlbumSelection>(
            predicate: #Predicate { item in
                item.localIdentifier == identifier
            }
        )

        let existing = try modelContext.fetch(descriptor).first

        if let existing {
            // Update existing
            existing.name = selection.albumName
            existing.selectionType = selection.selectionType == .included ? .included : .excluded
            existing.creationDate = selection.createdAt
        } else {
            // Create new
            let newSelection = BackupAlbumSelection(
                localIdentifier: selection.albumIdentifier,
                name: selection.albumName,
                selectionType: selection.selectionType == .included ? .included : .excluded,
                creationDate: selection.createdAt
            )
            modelContext.insert(newSelection)
        }

        try modelContext.save()
    }

    public func deleteSelection(withId id: String) async throws {
        let descriptor = FetchDescriptor<BackupAlbumSelection>(
            predicate: #Predicate { item in
                item.localIdentifier == id
            }
        )

        if let selection = try modelContext.fetch(descriptor).first {
            modelContext.delete(selection)
            try modelContext.save()
        }
    }

    public func findSelection(byAlbumId albumId: String) async throws -> BackupAlbumSelectionEntity? {
        let descriptor = FetchDescriptor<BackupAlbumSelection>(
            predicate: #Predicate { item in
                item.localIdentifier == albumId
            }
        )

        guard let selection = try modelContext.fetch(descriptor).first else {
            return nil
        }

        return BackupAlbumSelectionEntity(
            id: selection.localIdentifier,
            albumIdentifier: selection.localIdentifier,
            albumName: selection.name,
            selectionType: selection.selectionType == .included ? .included : .excluded,
            createdAt: selection.creationDate
        )
    }

    public func clearAllSelections() async throws {
        let descriptor = FetchDescriptor<BackupAlbumSelection>()
        let selections = try modelContext.fetch(descriptor)

        for selection in selections {
            modelContext.delete(selection)
        }

        try modelContext.save()
    }
}
