//
//  BackupViewModel.swift
//  Photonic
//
//  Interface Layer - Backup Management View Model
//

import Foundation
import SwiftUI

/// View model for the backup configuration and monitoring screen
///
/// Manages the backup workflow including album selection, progress monitoring,
/// and backup history. Coordinates between the backup use case and the UI
/// to provide real-time backup status and control.
@MainActor
final class BackupViewModel: ObservableObject {
    // MARK: - Dependencies

    private let backupService: BackupServiceProtocol
    private let albumRepository: AlbumRepository

    // MARK: - Published Properties

    /// All available albums for backup
    @Published var availableAlbums: [Album] = []

    /// Albums selected for backup
    @Published var selectedAlbumIds: Set<String> = []

    /// Current backup progress
    @Published var backupProgress: BackupProgress?

    /// Indicates if a backup is currently running
    @Published var isBackupRunning = false

    /// Indicates if the backup is paused
    @Published var isBackupPaused = false

    /// Loading state for albums
    @Published var isLoadingAlbums = false

    /// Error message for display
    @Published var errorMessage: String?

    /// Backup history/statistics
    @Published var totalItemsBackedUp = 0
    @Published var totalBytesUploaded: Int64 = 0
    @Published var lastBackupDate: Date?

    // MARK: - Private Properties

    private var backupTask: Task<Void, Never>?

    // MARK: - Computed Properties

    /// Albums that are currently selected for backup
    var selectedAlbums: [Album] {
        availableAlbums.filter { selectedAlbumIds.contains($0.id) }
    }

    /// Progress percentage (0-100)
    var progressPercentage: Double {
        guard let progress = backupProgress,
              progress.totalItems > 0
        else { return 0 }
        return Double(progress.processedItems) / Double(progress.totalItems) * 100
    }

    /// Formatted progress text
    var progressText: String {
        guard let progress = backupProgress else { return "No backup in progress" }
        return "\(progress.processedItems) / \(progress.totalItems) items"
    }

    /// Current status message
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
        case .failed:
            return "Backup failed"
        case .canceled:
            return "Backup canceled"
        case .idle:
            return "Ready to backup"
        }
    }

    // MARK: - Initialization

    init(backupService: BackupServiceProtocol, albumRepository: AlbumRepository) {
        self.backupService = backupService
        self.albumRepository = albumRepository
    }

    // MARK: - Public Methods

    /// Loads available albums from the repository
    func loadAlbums() async {
        isLoadingAlbums = true
        errorMessage = nil

        do {
            availableAlbums = try await albumRepository.fetchAll()
            isLoadingAlbums = false
        } catch {
            isLoadingAlbums = false
            errorMessage = "Failed to load albums: \(error.localizedDescription)"
        }
    }

    /// Toggles selection of an album for backup
    func toggleAlbumSelection(_ albumId: String) {
        if selectedAlbumIds.contains(albumId) {
            selectedAlbumIds.remove(albumId)
        } else {
            selectedAlbumIds.insert(albumId)
        }
    }

    /// Selects all albums for backup
    func selectAllAlbums() {
        selectedAlbumIds = Set(availableAlbums.map(\.id))
    }

    /// Deselects all albums
    func deselectAllAlbums() {
        selectedAlbumIds.removeAll()
    }

    /// Starts the backup process for selected albums
    func startBackup() async {
        guard !selectedAlbums.isEmpty else {
            errorMessage = "Please select at least one album to backup"
            return
        }

        // Cancel any existing backup
        backupTask?.cancel()

        backupTask = Task {
            isBackupRunning = true
            isBackupPaused = false
            errorMessage = nil

            do {
                // Convert selected albums to BackupAlbumSelectionEntity
                let selections = selectedAlbums.map { album in
                    BackupAlbumSelectionEntity(
                        albumIdentifier: album.id,
                        albumName: album.name,
                        selectionType: .included
                    )
                }

                let progressStream = try await backupService.startBackup(for: selections)

                for try await progress in progressStream {
                    guard !Task.isCancelled else { break }

                    backupProgress = progress

                    // Update statistics
                    if progress.status == .completed {
                        totalItemsBackedUp = progress.processedItems
                        lastBackupDate = Date()
                    }

                    // Handle errors in progress
                    if !progress.errors.isEmpty {
                        errorMessage = progress.errors.last?.message
                    }
                }

                isBackupRunning = false
            } catch {
                if !Task.isCancelled {
                    isBackupRunning = false
                    errorMessage = "Backup failed: \(error.localizedDescription)"
                }
            }
        }
    }

    /// Pauses the current backup
    func pauseBackup() async {
        await backupService.pauseBackup()
        isBackupPaused = true

        if var progress = backupProgress {
            progress.status = .paused
            backupProgress = progress
        }
    }

    /// Resumes a paused backup
    func resumeBackup() async {
        await backupService.resumeBackup()
        isBackupPaused = false

        if var progress = backupProgress {
            progress.status = .uploading
            backupProgress = progress
        }
    }

    /// Cancels the current backup
    func cancelBackup() async {
        backupTask?.cancel()
        await backupService.cancelBackup()

        isBackupRunning = false
        isBackupPaused = false

        if var progress = backupProgress {
            progress.status = .canceled
            backupProgress = progress
        }
    }

    /// Resets the backup state
    func resetBackup() {
        backupProgress = nil
        errorMessage = nil
    }
}
