import Dependencies
import Foundation
import PhotonicAPI
import PhotonicCore

enum UploadError: Error, Sendable {
    case serverNotConfigured
    case notAuthenticated
}

/// Live upload implementation: loads the asset's original data through the
/// photo-library client and posts it via `UploadAPI`. The durable queue plus
/// relaunch recovery (R12) carry the correctness guarantees.
extension UploadClient: DependencyKey {
    static var liveValue: UploadClient {
        UploadClient(upload: { job in
            @Dependency(PhotoLibraryClient.self) var photos
            @Dependency(ServerConfigurationClient.self) var serverConfiguration
            @Dependency(AuthClient.self) var auth

            let data = try await photos.loadData(job.mediaID)
            guard let configuration = await serverConfiguration.load() else {
                throw UploadError.serverNotConfigured
            }
            guard let token = await auth.restoreSession()?.accessToken.value else {
                throw UploadError.notAuthenticated
            }

            try await UploadAPI.createMedium(
                serverURL: configuration.serverURL.rawValue,
                accessToken: token,
                filename: job.filename ?? job.mediaID,
                dateTaken: job.dateTaken,
                data: data
            )
        })
    }
}
