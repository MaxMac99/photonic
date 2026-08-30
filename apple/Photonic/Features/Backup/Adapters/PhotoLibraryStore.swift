import Dependencies
import Foundation
import PhotonicCore
import Photos

extension PhotoLibraryClient: DependencyKey {
    static var liveValue: PhotoLibraryClient {
        PhotoLibraryClient(
            requestAccess: {
                let status = await PHPhotoLibrary.requestAuthorization(for: .readWrite)
                return status == .authorized || status == .limited
            },
            fetchAlbums: {
                var albums: [PhotoAlbum] = []

                let userAlbums = PHAssetCollection.fetchAssetCollections(
                    with: .album,
                    subtype: .any,
                    options: nil
                )
                userAlbums.enumerateObjects { collection, _, _ in
                    albums.append(collection.asPhotoAlbum)
                }

                let recents = PHAssetCollection.fetchAssetCollections(
                    with: .smartAlbum,
                    subtype: .smartAlbumUserLibrary,
                    options: nil
                )
                recents.enumerateObjects { collection, _, _ in
                    albums.append(collection.asPhotoAlbum)
                }

                return albums
            },
            pendingMedia: { albumID in
                guard let collection = Self.collection(for: albumID) else { return [] }
                let assets = PHAsset.fetchAssets(in: collection, options: nil)
                var pending: [PendingMedia] = []
                assets.enumerateObjects { asset, _, _ in
                    pending.append(
                        PendingMedia(
                            id: asset.localIdentifier,
                            type: Self.mediumType(of: asset),
                            filename: PHAssetResource.assetResources(for: asset)
                                .first(where: \.isOriginal)?.originalFilename,
                            dateTaken: asset.creationDate
                        )
                    )
                }
                return pending
            },
            loadData: { mediaID in
                try await Self.loadOriginalData(of: mediaID)
            }
        )
    }

    // MARK: - Private

    private static func collection(for identifier: String) -> PHAssetCollection? {
        PHAssetCollection.fetchAssetCollections(
            withLocalIdentifiers: [identifier],
            options: nil
        ).firstObject
    }

    private static func mediumType(of asset: PHAsset) -> MediumType? {
        switch asset.mediaType {
        case .image:
            asset.mediaSubtypes.contains(.photoLive) ? .livePhoto : .photo
        case .video:
            .video
        default:
            nil
        }
    }

    private static func loadOriginalData(of mediaID: String) async throws -> Data {
        guard
            let asset = PHAsset.fetchAssets(
                withLocalIdentifiers: [mediaID],
                options: nil
            ).firstObject
        else { throw PhotoLibraryError.assetNotFound }

        let resources = PHAssetResource.assetResources(for: asset)
        guard let resource = resources.first(where: \.isOriginal) ?? resources.first else {
            throw PhotoLibraryError.resourceNotFound
        }

        return try await withCheckedThrowingContinuation { continuation in
            var data = Data()
            let options = PHAssetResourceRequestOptions()
            options.isNetworkAccessAllowed = true
            PHAssetResourceManager.default().requestData(
                for: resource,
                options: options
            ) { chunk in
                data.append(chunk)
            } completionHandler: { error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: data)
                }
            }
        }
    }
}

enum PhotoLibraryError: Error, Sendable {
    case assetNotFound
    case resourceNotFound
}

private extension PHAssetCollection {
    var asPhotoAlbum: PhotoAlbum {
        let count = estimatedAssetCount != NSNotFound ? estimatedAssetCount : 0
        return PhotoAlbum(
            id: localIdentifier,
            name: localizedTitle ?? "Unknown",
            assetCount: count
        )
    }
}

private extension PHAssetResource {
    var isOriginal: Bool {
        type == .photo || type == .video || type == .fullSizePhoto || type == .fullSizeVideo
    }
}
