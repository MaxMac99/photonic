//
//  MediaRepository.swift
//  Photonic
//
//  Domain Repository Protocol
//

import Foundation

public protocol MediaRepository {
    func listAlbums() async throws -> [Album]
    func listMedia(in albums: [Album]) async throws -> AsyncThrowingStream<MediaItem, Error>
    func fetchMedia(albumId: String?, startDate: Date?, endDate: Date?, page: Int, pageSize: Int) async throws -> [MediaItem]
    func upload(_ item: MediaItem, data: Data) async throws -> UploadResult
    func getMediaData(for item: MediaItem) async throws -> Data
}

public struct UploadResult: Equatable {
    public let mediaId: String
    public let success: Bool
    public let error: String?
    
    public init(mediaId: String, success: Bool, error: String? = nil) {
        self.mediaId = mediaId
        self.success = success
        self.error = error
    }
}