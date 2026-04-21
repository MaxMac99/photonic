//
//  PreviewDependencies.swift
//  Photonic
//
//  Preview Helpers and Mock Dependencies
//

import Foundation
import OpenAPIURLSession
import SwiftUI

// MARK: - Preview Dependencies Container

enum PreviewDependencies {

    // MARK: - Mock Repositories

    static let mockAuthRepository = MockAuthRepository()
    static let mockMediaRepository = MockMediaRepository()
    static let mockAlbumRepository = MockAlbumRepository()
    static let mockUserRepository = MockUserRepository()
    static let mockServerConfigRepository = MockServerConfigurationRepository()

    // MARK: - Mock Use Cases and Services

    static let mockDiscoverServerUseCase = MockDiscoverServerUseCase()
    static let mockBackupService = MockBackupService()

    // MARK: - Helper Methods

    static func createMockClient() -> APIProtocol {
        // Return the MockAPI from the Preview Content folder
        return MockAPI()
    }

    static func createMockCompositionRoot() -> CompositionRoot {
        return CompositionRoot(
            serverInfo: ServerInfo(
                serverUrl: URL(string: "https://photonic.example.com")!,
                clientId: "mock-client",
                tokenUrl: URL(string: "https://photonic.example.com/oauth/token")!,
                authorizationUrl: URL(string: "https://photonic.example.com/oauth/authorize")!
            ))
    }

    // MARK: - Sample Data

    static let sampleServerConfig = ServerConfiguration(
        serverUrl: URL(string: "https://photonic.example.com")!,
        clientId: "sample-client-id",
        tokenUrl: URL(string: "https://photonic.example.com/oauth/token")!,
        authorizationUrl: URL(string: "https://photonic.example.com/oauth/authorize")!
    )!

    static let sampleAccessToken = AccessToken(
        value: "sample-access-token",
        expiresAt: Date().addingTimeInterval(3600),
        scopes: ["openid", "profile", "email", "offline_access"]
    )

    static let sampleRefreshToken = RefreshToken(
        value: "sample-refresh-token",
        issuedAt: Date()
    )

    static let sampleUser = UserAccount(
        id: "user-123",
        email: "john.doe@example.com",
        name: "John Doe",
        givenName: "John",
        familyName: "Doe",
        nickname: "JD",
        preferredUsername: "johndoe",
        profileUrl: "https://photonic.example.com/users/johndoe",
        pictureUrl: "https://photonic.example.com/users/johndoe/avatar.jpg",
        emailVerified: true,
        quota: UserAccount.Quota(totalBytes: 10_737_418_240, usedBytes: 5_368_709_120),
        createdAt: Date().addingTimeInterval(-86400 * 30)
    )

    static let sampleAlbums = [
        Album(
            id: "album-1",
            name: "Vacation 2024",
            createdAt: Date().addingTimeInterval(-86400 * 7),
            updatedAt: Date().addingTimeInterval(-86400 * 2),
            mediaCount: 145,
            coverMediaId: "media-1",
            isShared: false,
            ownerUserId: "user-123"
        ),
        Album(
            id: "album-2",
            name: "Family Photos",
            createdAt: Date().addingTimeInterval(-86400 * 30),
            updatedAt: Date().addingTimeInterval(-86400 * 1),
            mediaCount: 523,
            coverMediaId: "media-10",
            isShared: true,
            ownerUserId: "user-123"
        ),
        Album(
            id: "album-3",
            name: "Screenshots",
            createdAt: Date().addingTimeInterval(-86400 * 60),
            updatedAt: Date(),
            mediaCount: 89,
            coverMediaId: nil,
            isShared: false,
            ownerUserId: "user-123"
        ),
    ]

    static let sampleMediaItems = [
        MediaItem(
            id: "media-1",
            checksum: "abc123def456",
            createdAt: Date().addingTimeInterval(-86400 * 5),
            mimeType: "image/jpeg",
            sizeInBytes: 2_456_789,
            width: 4032,
            height: 3024,
            duration: nil,
            location: MediaItem.Location(latitude: 37.7749, longitude: -122.4194, altitude: 15.5),
            albumIds: ["album-1"]
        ),
        MediaItem(
            id: "media-2",
            checksum: "def789ghi012",
            createdAt: Date().addingTimeInterval(-86400 * 3),
            mimeType: "video/mp4",
            sizeInBytes: 45_678_901,
            width: 1920,
            height: 1080,
            duration: 30.5,
            location: nil,
            albumIds: ["album-1", "album-2"]
        ),
        MediaItem(
            id: "media-3",
            checksum: "jkl345mno678",
            createdAt: Date().addingTimeInterval(-86400 * 1),
            mimeType: "image/heic",
            sizeInBytes: 1_234_567,
            width: 3024,
            height: 4032,
            duration: nil,
            location: MediaItem.Location(latitude: 40.7128, longitude: -74.0060, altitude: 10.0),
            albumIds: ["album-2"]
        ),
    ]
}

// MARK: - Mock Repositories

final class MockAuthRepository: AuthRepository {
    var shouldSucceed = true
    var tokens: (access: AccessToken, refresh: RefreshToken)? = (
        PreviewDependencies.sampleAccessToken,
        PreviewDependencies.sampleRefreshToken
    )

    func signInInteractive() async throws -> (access: AccessToken, refresh: RefreshToken) {
        guard shouldSucceed else {
            throw AuthError.signInFailed("Mock sign in failed")
        }
        return (PreviewDependencies.sampleAccessToken, PreviewDependencies.sampleRefreshToken)
    }

    func signOut() async throws {
        tokens = nil
    }

    func getUserAccount() async throws -> UserAccount {
        return PreviewDependencies.sampleUser
    }
}

final class MockMediaRepository: MediaRepository {
    func listAlbums() async throws -> [Album] {
        return PreviewDependencies.sampleAlbums
    }

    func listMedia(in albums: [Album]) async throws -> AsyncThrowingStream<MediaItem, Error> {
        AsyncThrowingStream { continuation in
            Task {
                for item in PreviewDependencies.sampleMediaItems {
                    continuation.yield(item)
                }
                continuation.finish()
            }
        }
    }

    func fetchMedia(albumId: String?, startDate: Date?, endDate: Date?, page: Int, pageSize: Int)
        async throws -> [MediaItem]
    {
        return PreviewDependencies.sampleMediaItems
    }

    func upload(_ item: MediaItem, data: Data) async throws -> UploadResult {
        return UploadResult(mediaId: item.id, success: true, error: nil)
    }

    func getMediaData(for item: MediaItem) async throws -> Data {
        return Data()
    }
}

final class MockAlbumRepository: AlbumRepository {
    func fetchAll() async throws -> [Album] {
        return PreviewDependencies.sampleAlbums
    }

    func fetch(id: String) async throws -> Album {
        guard let album = PreviewDependencies.sampleAlbums.first(where: { $0.id == id }) else {
            throw DomainError.notFound("Album")
        }
        return album
    }

    func create(name: String) async throws -> Album {
        return Album(
            id: UUID().uuidString,
            name: name,
            createdAt: Date(),
            updatedAt: Date(),
            ownerUserId: "user-123"
        )
    }

    func update(_ album: Album) async throws -> Album {
        return album
    }

    func delete(id: String) async throws {
        // Mock implementation
    }

    func addMedia(albumId: String, mediaIds: [String]) async throws {
        // Mock implementation
    }

    func removeMedia(albumId: String, mediaIds: [String]) async throws {
        // Mock implementation
    }
}

final class MockUserRepository: UserRepository {
    func getUserStats() async throws -> UserStats {
        return UserStats(
            albums: 3,
            media: 757,
            quota: 100_000_000_000,
            quotaUsed: 12_000_000_000
        )
    }
}

final class MockServerConfigurationRepository: ServerConfigurationRepository {
    var configuration: ServerConfiguration? = PreviewDependencies.sampleServerConfig

    func getCurrentConfiguration() async throws -> ServerConfiguration? {
        return configuration
    }

    func saveConfiguration(_ configuration: ServerConfiguration) async throws {
        self.configuration = configuration
    }

    func deleteConfiguration() async throws {
        self.configuration = nil
    }

    func discoverServerInfo(url: URL) async throws -> ServerDiscoveryInfo {
        return ServerDiscoveryInfo(
            clientId: "discovered-client-id",
            authorizeUrl: URL(string: "\(url.absoluteString)/oauth/authorize")!,
            tokenUrl: URL(string: "\(url.absoluteString)/oauth/token")!,
            serverVersion: "1.0.0"
        )
    }
}

// MARK: - Mock Use Cases and Services

final class MockBackupService: BackupServiceProtocol {
    func startBackup(for selections: [BackupAlbumSelectionEntity]) async throws -> AsyncThrowingStream<BackupProgress, Error> {
        AsyncThrowingStream { continuation in
            Task {
                let total = 10
                for i in 0...total {
                    continuation.yield(
                        BackupProgress(
                            totalItems: total,
                            processedItems: i,
                            currentItem: i < total
                                ? PreviewDependencies.sampleMediaItems.first : nil,
                            status: i < total ? .uploading : .completed,
                            errors: []
                        ))
                    try? await Task.sleep(nanoseconds: 500_000_000)
                }
                continuation.finish()
            }
        }
    }

    func pauseBackup() async {
        // Mock implementation
    }

    func resumeBackup() async {
        // Mock implementation
    }

    func cancelBackup() async {
        // Mock implementation
    }
}
