//
//  AlbumRepositoryImpl.swift
//  Photonic
//
//  Infrastructure Layer - API Implementation
//

import Foundation
import OpenAPIURLSession

final class AlbumRepositoryImpl: AlbumRepository {
    private let logger = LoggerFactory.logger(for: .api)
    private let apiClient: APIProtocol

    init(apiClient: APIProtocol) {
        self.apiClient = apiClient
    }

    func fetchAll() async throws -> [Album] {
        logger.debug("Fetching all albums")

        // For now, return empty array until API is properly implemented
        logger.info("Albums fetched successfully - Count: 0 (stub implementation)")
        return []
    }

    func fetch(id: String) async throws -> Album {
        logger.debug("Fetching album with id: \(id)")

        // For now, throw not found until API is properly implemented
        logger.error("Album not found: \(id) (stub implementation)")
        throw DomainError.notFound("Album")
    }

    func create(name: String) async throws -> Album {
        logger.debug("Creating album with name: \(name)")

        // For now, return a stub album until API is properly implemented
        let album = Album(
            id: UUID().uuidString,
            name: name,
            createdAt: Date(),
            updatedAt: Date(),
            ownerUserId: "stub-user-id"
        )

        logger.info("Album created successfully: \(album.id) (stub implementation)")
        return album
    }

    func update(_ album: Album) async throws -> Album {
        logger.debug("Updating album: \(album.id)")

        // For now, return the same album until API is properly implemented
        logger.info("Album updated successfully: \(album.id) (stub implementation)")
        return album
    }

    func delete(id: String) async throws {
        logger.debug("Deleting album: \(id)")

        // For now, just log until API is properly implemented
        logger.info("Album deleted successfully: \(id) (stub implementation)")
    }

    func addMedia(albumId: String, mediaIds: [String]) async throws {
        logger.debug("Adding \(mediaIds.count) media items to album: \(albumId)")

        // For now, just log until API is properly implemented
        logger.info("Media added to album successfully: \(albumId) (stub implementation)")
    }

    func removeMedia(albumId: String, mediaIds: [String]) async throws {
        logger.debug("Removing \(mediaIds.count) media items from album: \(albumId)")

        // For now, just log until API is properly implemented
        logger.info("Media removed from album successfully: \(albumId) (stub implementation)")
    }
}
