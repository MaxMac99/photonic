import ComposableArchitecture
import Dependencies
import Foundation

/// Typed interface for uploading one job's media to the server.
@DependencyClient
struct UploadClient: Sendable {
    var upload: @Sendable (UploadJob) async throws -> Void
}

extension DependencyValues {
    var uploadClient: UploadClient {
        get { self[UploadClient.self] }
        set { self[UploadClient.self] = newValue }
    }
}
