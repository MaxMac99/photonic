//
//  BackupService.swift
//  Photonic
//
//  Infrastructure Layer - Backup Service
//

import CoreLocation
import Foundation
import Photos

protocol BackupServiceProtocol {
    func startBackup(for selections: [BackupAlbumSelectionEntity]) async throws
        -> AsyncThrowingStream<BackupProgress, Error>
    func pauseBackup() async
    func resumeBackup() async
    func cancelBackup() async
}

struct BackupProgress: Equatable {
    let totalItems: Int
    let processedItems: Int
    let currentItem: MediaItem?
    var status: BackupStatus
    let errors: [BackupError]

    var progressPercentage: Double {
        guard totalItems > 0 else { return 0 }
        return Double(processedItems) / Double(totalItems) * 100
    }
}

enum BackupStatus: Equatable {
    case idle
    case preparing
    case uploading
    case paused
    case completed
    case failed(String)
    case canceled
}

struct BackupError: Equatable {
    let mediaId: String
    let message: String
    let timestamp: Date

    init(mediaId: String, message: String, timestamp: Date = Date()) {
        self.mediaId = mediaId
        self.message = message
        self.timestamp = timestamp
    }
}

final class BackupService: BackupServiceProtocol {
    private static let logger = LoggerFactory.logger(for: .application)

    private let mediaRepository: MediaRepository
    private let albumRepository: AlbumRepository
    private let photoLibraryAdapter: PhotoLibraryAdapter

    private var isPaused = false
    private var isCancelled = false

    init(
        mediaRepository: MediaRepository,
        albumRepository: AlbumRepository,
        photoLibraryAdapter: PhotoLibraryAdapter
    ) {
        self.mediaRepository = mediaRepository
        self.albumRepository = albumRepository
        self.photoLibraryAdapter = photoLibraryAdapter
    }

    func startBackup(for selections: [BackupAlbumSelectionEntity]) async throws
        -> AsyncThrowingStream<BackupProgress, Error>
    {
        Self.logger.info("Starting backup for \(selections.count) album selections")
        isPaused = false
        isCancelled = false

        return AsyncThrowingStream { continuation in
            Task {
                do {
                    // Emit preparing status
                    continuation.yield(
                        BackupProgress(
                            totalItems: 0,
                            processedItems: 0,
                            currentItem: nil,
                            status: .preparing,
                            errors: []
                        )
                    )

                    // Get albums to backup (only included ones, exclude excluded ones)
                    let includedSelections = selections.filter { $0.selectionType == .included }
                    _ = Set(
                        selections.filter { $0.selectionType == .excluded }.map(\.albumIdentifier)
                    )

                    // Convert to Album entities
                    var albumsToBackup: [Album] = []
                    for selection in includedSelections {
                        if let collection = getCollection(for: selection.albumIdentifier) {
                            let album = Album(
                                id: collection.localIdentifier,
                                name: collection.localizedTitle ?? "Unknown",
                                assetCount: getAssetCount(for: collection)
                            )
                            albumsToBackup.append(album)
                        }
                    }

                    // Get all media items from selected albums
                    var allMedia: [MediaItem] = []
                    for album in albumsToBackup {
                        if let collection = getCollection(for: album.id) {
                            let mediaItems = try await getMediaItems(from: collection)
                            allMedia.append(contentsOf: mediaItems)
                        }
                    }

                    // Get existing media on server to avoid duplicates
                    let existingMedia = try await mediaRepository.fetchMedia(
                        albumId: nil,
                        startDate: nil,
                        endDate: nil,
                        page: 1,
                        pageSize: 10000
                    )

                    let existingChecksums = Set(existingMedia.map(\.checksum))
                    let mediaToUpload = allMedia.filter { !existingChecksums.contains($0.checksum) }

                    Self.logger.info("Found \(mediaToUpload.count) new items to upload")

                    var errors: [BackupError] = []

                    // Upload each media item
                    for (index, item) in mediaToUpload.enumerated() {
                        // Check for pause/cancel
                        while isPaused, !isCancelled {
                            try await Task.sleep(nanoseconds: 100_000_000) // 0.1 second
                        }

                        if isCancelled {
                            continuation.yield(
                                BackupProgress(
                                    totalItems: mediaToUpload.count,
                                    processedItems: index,
                                    currentItem: nil,
                                    status: .canceled,
                                    errors: errors
                                )
                            )
                            break
                        }

                        // Emit progress
                        continuation.yield(
                            BackupProgress(
                                totalItems: mediaToUpload.count,
                                processedItems: index,
                                currentItem: item,
                                status: .uploading,
                                errors: errors
                            )
                        )

                        // Get media data and upload
                        do {
                            let data = try await photoLibraryAdapter.getData(for: item)
                            _ = try await mediaRepository.upload(item, data: data)
                            Self.logger.debug("Successfully uploaded item: \(item.id)")
                        } catch {
                            Self.logger.error(
                                "Failed to upload item \(item.id): \(error.localizedDescription)"
                            )
                            errors.append(
                                BackupError(
                                    mediaId: item.id,
                                    message: error.localizedDescription,
                                    timestamp: Date()
                                )
                            )
                        }
                    }

                    // Emit completion
                    let finalStatus: BackupStatus =
                        errors.isEmpty
                            ? .completed
                            : .failed("Backup completed with \(errors.count) errors")

                    continuation.yield(
                        BackupProgress(
                            totalItems: mediaToUpload.count,
                            processedItems: mediaToUpload.count,
                            currentItem: nil,
                            status: finalStatus,
                            errors: errors
                        )
                    )

                    Self.logger.info("Backup completed with \(errors.count) errors")
                    continuation.finish()
                } catch {
                    Self.logger.error("Backup failed: \(error.localizedDescription)")
                    continuation.finish(throwing: error)
                }
            }
        }
    }

    func pauseBackup() async {
        Self.logger.info("Pausing backup")
        isPaused = true
    }

    func resumeBackup() async {
        Self.logger.info("Resuming backup")
        isPaused = false
    }

    func cancelBackup() async {
        Self.logger.info("Canceling backup")
        isCancelled = true
    }

    // MARK: - Private Helpers

    private func getCollection(for identifier: String) -> PHAssetCollection? {
        let result = PHAssetCollection.fetchAssetCollections(
            withLocalIdentifiers: [identifier],
            options: nil
        )
        return result.firstObject
    }

    private func getAssetCount(for collection: PHAssetCollection) -> Int {
        let estimation = collection.estimatedAssetCount
        if estimation != NSNotFound {
            return estimation
        }

        let result = PHAsset.fetchAssets(in: collection, options: nil)
        return result.count
    }

    private func getMediaItems(from collection: PHAssetCollection) async throws -> [MediaItem] {
        let assets = PHAsset.fetchAssets(in: collection, options: nil)
        var mediaItems: [MediaItem] = []

        for i in 0 ..< assets.count {
            let asset = assets.object(at: i)

            // Create MediaItem from PHAsset
            let location: MediaItem.Location? = asset.location.map { clLocation in
                MediaItem.Location(
                    latitude: clLocation.coordinate.latitude,
                    longitude: clLocation.coordinate.longitude,
                    altitude: clLocation.altitude
                )
            }

            let mediaItem = MediaItem(
                id: asset.localIdentifier,
                checksum: asset.localIdentifier, // Using identifier as checksum for now
                createdAt: asset.creationDate ?? Date(),
                location: location
            )

            mediaItems.append(mediaItem)
        }

        return mediaItems
    }
}
