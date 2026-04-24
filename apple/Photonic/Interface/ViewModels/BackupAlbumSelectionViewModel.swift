//
//  BackupAlbumSelectionViewModel.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import Foundation
import Photos
import SwiftUI

@MainActor
final class BackupAlbumSelectionViewModel: ObservableObject {
    private static let logger = LoggerFactory.logger(for: .ui)
    @Published var collections: [PHAssetCollection] = []
    @Published var selections: [BackupAlbumSelectionEntity] = []
    @Published var isLoading = false
    @Published var showError = false
    @Published var errorMessage = ""

    // Backup-related properties
    @Published var isBackupInProgress = false
    @Published var backupProgress: BackupProgress?
    @Published var showBackupSheet = false

    private let backupSelectionRepository: BackupSelectionRepository
    private let photoLibraryAdapter: PhotoLibraryAdapter
    private let backupService: BackupServiceProtocol?
    private var backupTask: Task<Void, Never>?

    init(
        backupSelectionRepository: BackupSelectionRepository,
        photoLibraryAdapter: PhotoLibraryAdapter = PhotoLibraryAdapter(),
        backupService: BackupServiceProtocol? = nil
    ) {
        self.backupSelectionRepository = backupSelectionRepository
        self.photoLibraryAdapter = photoLibraryAdapter
        self.backupService = backupService
    }

    func loadData() async {
        isLoading = true
        defer { isLoading = false }

        // Load albums from Photos library
        await loadAlbums()

        // Load saved selections
        do {
            selections = try await backupSelectionRepository.fetchSelections()
            Self.logger.info("Loaded \(selections.count) saved selections")
        } catch {
            Self.logger.error("Failed to load selections: \(error.localizedDescription)")
            showError = true
            errorMessage = "Failed to load saved selections"
        }
    }

    private func loadAlbums() async {
        Self.logger.debug("Loading albums from Photos library")

        var allCollections: [PHAssetCollection] = []

        // Fetch smart albums
        let smartCollections = PHAssetCollection.fetchAssetCollections(
            with: .smartAlbum,
            subtype: .any,
            options: nil
        )
        allCollections.append(
            contentsOf: smartCollections.objects(
                at: IndexSet(integersIn: 0 ..< smartCollections.count)
            )
        )

        // Fetch user collections
        let userCollections = PHCollection.fetchTopLevelUserCollections(with: nil)
        let assetCollections = userCollections.objects(
            at: IndexSet(integersIn: 0 ..< userCollections.count)
        )
        .compactMap { $0 as? PHAssetCollection }

        allCollections.append(contentsOf: assetCollections)

        // Filter out empty collections
        collections = allCollections.filter { collection in
            let assets = PHAsset.fetchAssets(in: collection, options: nil)
            return !assets.isEmpty
        }

        Self.logger.info("Loaded \(collections.count) albums")
    }

    func selection(for collection: PHAssetCollection) -> BackupAlbumSelectionEntity? {
        selections.first { $0.albumIdentifier == collection.localIdentifier }
    }

    func updateSelection(
        for collection: PHAssetCollection,
        type: AlbumSelectionType?
    ) async {
        let identifier = collection.localIdentifier
        let name = collection.localizedTitle ?? "Unknown"

        do {
            if let type {
                // Create or update selection
                let selection = BackupAlbumSelectionEntity(
                    albumIdentifier: identifier,
                    albumName: name,
                    selectionType: type,
                    createdAt: Date()
                )

                try await backupSelectionRepository.saveSelection(selection)

                // Update local state
                selections.removeAll { $0.albumIdentifier == identifier }
                selections.append(selection)

                Self.logger.info("Updated selection for album \(name): \(type)")
            } else {
                // Remove selection
                if let existing = selections.first(where: { $0.albumIdentifier == identifier }) {
                    try await backupSelectionRepository.deleteSelection(withId: existing.id)
                    selections.removeAll { $0.id == existing.id }
                    Self.logger.info("Removed selection for album \(name)")
                }
            }
        } catch {
            Self.logger.error("Failed to update selection: \(error.localizedDescription)")
            showError = true
            errorMessage = "Failed to save selection"
        }
    }

    func removeSelection(_ selection: BackupAlbumSelectionEntity) async {
        do {
            try await backupSelectionRepository.deleteSelection(withId: selection.id)
            selections.removeAll { $0.id == selection.id }
            Self.logger.info("Removed selection: \(selection.albumName)")
        } catch {
            Self.logger.error("Failed to remove selection: \(error.localizedDescription)")
            showError = true
            errorMessage = "Failed to remove selection"
        }
    }

    func getAssetCount(for collection: PHAssetCollection) -> Int {
        let estimation = collection.estimatedAssetCount
        if estimation != NSNotFound {
            return estimation
        }

        let result = PHAsset.fetchAssets(in: collection, options: nil)
        return result.count
    }

    // MARK: - Backup Methods

    func startBackup() async {
        guard let backupService else {
            Self.logger.error("Backup service not available")
            showError = true
            errorMessage = "Backup service not available"
            return
        }

        guard !selections.isEmpty else {
            showError = true
            errorMessage = "Please select at least one album for backup"
            return
        }

        // Cancel any existing backup
        backupTask?.cancel()

        backupTask = Task {
            isBackupInProgress = true
            errorMessage = ""

            do {
                let progressStream = try await backupService.startBackup(for: selections)

                for try await progress in progressStream {
                    guard !Task.isCancelled else { break }

                    backupProgress = progress

                    // Handle errors in progress
                    if !progress.errors.isEmpty {
                        errorMessage = progress.errors.last?.message ?? ""
                    }

                    // Check if completed or failed
                    switch progress.status {
                    case .completed, .failed, .canceled:
                        isBackupInProgress = false
                    default:
                        break
                    }
                }

                if !Task.isCancelled {
                    isBackupInProgress = false
                }
            } catch {
                if !Task.isCancelled {
                    isBackupInProgress = false
                    showError = true
                    errorMessage = "Backup failed: \(error.localizedDescription)"
                    Self.logger.error("Backup failed: \(error.localizedDescription)")
                }
            }
        }
    }

    func pauseBackup() async {
        guard let backupService else { return }
        await backupService.pauseBackup()
    }

    func resumeBackup() async {
        guard let backupService else { return }
        await backupService.resumeBackup()
    }

    func cancelBackup() async {
        backupTask?.cancel()

        guard let backupService else { return }
        await backupService.cancelBackup()

        isBackupInProgress = false
        backupProgress = nil
    }

    var hasSelectionsForBackup: Bool {
        selections.contains { $0.selectionType == .included }
    }

    var progressText: String {
        guard let progress = backupProgress else { return "No backup in progress" }
        return "\(progress.processedItems) / \(progress.totalItems) items"
    }

    var statusMessage: String {
        guard let progress = backupProgress else { return "Ready to backup" }

        switch progress.status {
        case .preparing:
            return "Preparing backup..."
        case .uploading:
            return "Uploading: \(progress.currentItem?.id ?? "...")"
        case .paused:
            return "Backup paused"
        case .completed:
            return "Backup completed successfully"
        case let .failed(message):
            return "Backup failed: \(message)"
        case .canceled:
            return "Backup canceled"
        case .idle:
            return "Ready to backup"
        }
    }
}
