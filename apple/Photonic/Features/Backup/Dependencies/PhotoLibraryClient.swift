import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicCore

/// Typed interface to the device photo library. PhotoKit usage is confined
/// to the adapter (R3); backup selection and upload data flow through here.
@DependencyClient
struct PhotoLibraryClient: Sendable {
    /// Requests read/write access; returns whether access was granted.
    var requestAccess: @Sendable () async -> Bool = { false }
    var fetchAlbums: @Sendable () async throws -> [PhotoAlbum]
    /// Lists every pending asset of an album.
    var pendingMedia: @Sendable (String) async throws -> [PendingMedia]
    /// Loads the original data of one asset.
    var loadData: @Sendable (String) async throws -> Data
}

extension DependencyValues {
    var photoLibraryClient: PhotoLibraryClient {
        get { self[PhotoLibraryClient.self] }
        set { self[PhotoLibraryClient.self] = newValue }
    }
}
