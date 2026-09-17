import Dependencies
import Foundation
import os
import PhotonicAPI
import PhotonicCore

enum UploadError: Error, Sendable {
    case serverNotConfigured
    case notAuthenticated
}

/// Live upload implementation: loads the asset's original data through the
/// photo-library client, creates the medium, then generates and uploads the
/// tiny/small/large thumbnail variants. Thumbnail failures are logged
/// per-variant without failing the already-successful backup job. The
/// durable queue plus relaunch recovery (R12) carry the correctness
/// guarantees.
extension UploadClient: DependencyKey {
    private static let logger = Logger(subsystem: "de.mvissing.photonic", category: "backup")

    static var liveValue: UploadClient {
        UploadClient(upload: { job in
            @Dependency(PhotoLibraryClient.self) var photos
            @Dependency(ThumbnailGenerator.self) var thumbnails
            @Dependency(ServerConfigurationClient.self) var serverConfiguration
            @Dependency(AuthClient.self) var auth

            let data = try await photos.loadData(job.mediaID)
            guard let configuration = await serverConfiguration.load() else {
                throw UploadError.serverNotConfigured
            }
            guard let token = await auth.restoreSession()?.accessToken.value else {
                throw UploadError.notAuthenticated
            }

            let mediumID = try await UploadAPI.createMedium(
                serverURL: configuration.serverURL.rawValue,
                accessToken: token,
                filename: job.filename ?? job.mediaID,
                dateTaken: job.dateTaken,
                data: data
            )

            guard let mediaType = job.mediaType, mediaType.isImage else { return }
            await uploadThumbnails(
                of: mediumID,
                originalFilename: job.filename ?? job.mediaID,
                originalData: data,
                serverURL: configuration.serverURL.rawValue,
                accessToken: token,
                generator: thumbnails
            )
        })
    }

    // MARK: - Thumbnails

    /// Generates all variants and uploads them as preview items. Thumbnail
    /// failures never fail the job: the original is already safe on the
    /// server, so each failure is logged and the remaining variants continue.
    private static func uploadThumbnails(
        of mediumID: UUID,
        originalFilename: String,
        originalData: Data,
        serverURL: URL,
        accessToken: String,
        generator: ThumbnailGenerator
    ) async {
        let thumbnails: [GeneratedThumbnail]
        do {
            thumbnails = try generator.generate(originalData)
        } catch {
            logger.error("Thumbnail generation failed: \(error)")
            return
        }

        let stem = (originalFilename as NSString).deletingPathExtension
        for thumbnail in thumbnails {
            do {
                _ = try await UploadAPI.addPreviewItem(
                    serverURL: serverURL,
                    accessToken: accessToken,
                    mediumID: mediumID,
                    variant: thumbnail.variant,
                    filename: "\(stem)_\(thumbnail.variant.rawValue).jpg",
                    width: thumbnail.width,
                    height: thumbnail.height,
                    data: thumbnail.data
                )
            } catch {
                logger.error(
                    "Thumbnail \(thumbnail.variant.rawValue, privacy: .public) upload failed: \(error)"
                )
            }
        }
    }
}

extension MediumType {
    /// Raster image types the client can downsample into JPEG variants.
    var isImage: Bool {
        switch self {
        case .photo, .livePhoto, .gif:
            true
        default:
            false
        }
    }
}
