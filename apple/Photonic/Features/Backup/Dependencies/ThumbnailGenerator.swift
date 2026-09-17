import ComposableArchitecture
import Dependencies
import Foundation

/// Typed interface for generating thumbnail variants from an image's
/// original data.
@DependencyClient
struct ThumbnailGenerator: Sendable {
    var generate: @Sendable (Data) throws -> [GeneratedThumbnail]
}

extension DependencyValues {
    var thumbnailGenerator: ThumbnailGenerator {
        get { self[ThumbnailGenerator.self] }
        set { self[ThumbnailGenerator.self] = newValue }
    }
}
