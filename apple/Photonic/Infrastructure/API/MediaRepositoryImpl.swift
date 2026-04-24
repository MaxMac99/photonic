//
//  MediaRepositoryImpl.swift
//  Photonic
//
//  Infrastructure Layer - API Implementation
//

import Foundation
import OpenAPIURLSession

final class MediaRepositoryImpl: MediaRepository {
    private let logger = LoggerFactory.logger(for: .api)
    private let apiClient: APIProtocol

    init(apiClient: APIProtocol) {
        self.apiClient = apiClient
    }

    func listAlbums() async throws -> [Album] {
        logger.debug("Fetching albums")

        // For now, return empty array until API is properly implemented
        logger.info("Albums fetched successfully - Count: 0 (stub implementation)")
        return []
    }

    func listMedia(in albums: [Album]) async throws -> AsyncThrowingStream<MediaItem, Error> {
        AsyncThrowingStream { continuation in
            Task {
                // For now, return empty stream until API is properly implemented
                logger.info("Media stream completed - Count: 0 (stub implementation)")
                continuation.finish()
            }
        }
    }

    func fetchMedia(albumId: String?, startDate: Date?, endDate: Date?, page: Int, pageSize: Int) async throws -> [MediaItem] {
        logger.debug("Fetching media - Album: \(albumId ?? "all"), Page: \(page)")

        // For now, return empty array until API is properly implemented
        logger.info("Media fetched successfully - Count: 0 (stub implementation)")
        return []
    }

    func upload(_ item: MediaItem, data: Data) async throws -> UploadResult {
        logger.debug("Uploading media item: \(item.id)")

        // For now, return success until API is properly implemented
        logger.info("Media uploaded successfully: \(item.id) (stub implementation)")
        return UploadResult(
            mediaId: item.id,
            success: true,
            error: nil
        )
    }

    func getMediaData(for item: MediaItem) async throws -> Data {
        logger.debug("Fetching media data for: \(item.id)")

        // For now, return empty data until API is properly implemented
        logger.info("Media data fetched successfully: \(item.id) (stub implementation)")
        return Data()
    }
}
