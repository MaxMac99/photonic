//
//  AlbumRepository.swift
//  Photonic
//
//  Domain Repository Protocol
//

import Foundation

public protocol AlbumRepository {
    func fetchAll() async throws -> [Album]
    func fetch(id: String) async throws -> Album
    func create(name: String) async throws -> Album
    func update(_ album: Album) async throws -> Album
    func delete(id: String) async throws
    func addMedia(albumId: String, mediaIds: [String]) async throws
    func removeMedia(albumId: String, mediaIds: [String]) async throws
}