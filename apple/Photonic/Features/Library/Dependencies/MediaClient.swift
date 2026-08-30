import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicAPI
import PhotonicCore

/// Typed interface to the server's media library, cursor-paged (R13).
@DependencyClient
struct MediaClient: Sendable {
    var fetchPage: @Sendable (MediaCursor?, Int) async throws -> MediaPage
}

enum MediaClientError: Error, Sendable {
    case serverNotConfigured
}

extension DependencyValues {
    var mediaClient: MediaClient {
        get { self[MediaClient.self] }
        set { self[MediaClient.self] = newValue }
    }
}
