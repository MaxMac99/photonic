//
//  BackupAlbumItemViewModel.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import Foundation
import OSLog
import Photos
import SwiftUI
import UIKit

@MainActor
public final class BackupAlbumItemViewModel: ObservableObject {
    private static let logger = Logger(
        subsystem: Bundle.main.bundleIdentifier!,
        category: String(describing: BackupAlbumItemViewModel.self)
    )

    @Published var thumbnailImage: UIImage?
    @Published var isLoadingImage = false
    @Published var assetCount = 0

    let collection: PHAssetCollection
    private let imageHeight: CGFloat = 124

    public init(collection: PHAssetCollection) {
        self.collection = collection
        assetCount = calculateAssetCount()
    }

    public var title: String {
        collection.localizedTitle ?? "Unknown"
    }

    public var identifier: String {
        collection.localIdentifier
    }

    private func calculateAssetCount() -> Int {
        let estimation = collection.estimatedAssetCount
        if estimation != NSNotFound {
            return estimation
        }

        let fetchOptions = PHFetchOptions()
        fetchOptions.includeHiddenAssets = true
        fetchOptions.includeAllBurstAssets = false

        let result = PHAsset.fetchAssets(in: collection, options: fetchOptions)
        return result.count
    }

    public func loadThumbnail() async {
        guard thumbnailImage == nil else { return }

        isLoadingImage = true
        defer { isLoadingImage = false }

        guard let asset = PHAsset.fetchKeyAssets(in: collection, options: nil)?.firstObject else {
            Self.logger.debug("No key asset found for collection \(identifier)")
            return
        }

        let targetSize = CGSize(width: imageHeight, height: imageHeight)

        Self.logger.debug("Loading thumbnail for \(identifier), asset size: \(asset.pixelWidth)x\(asset.pixelHeight)")

        await withCheckedContinuation { continuation in
            let options = PHImageRequestOptions()
            options.deliveryMode = .opportunistic
            options.isNetworkAccessAllowed = false
            options.resizeMode = .fast

            PHCachingImageManager.default().requestImage(
                for: asset,
                targetSize: targetSize,
                contentMode: .aspectFill,
                options: options
            ) { [weak self] image, info in
                guard let self else {
                    continuation.resume()
                    return
                }

                if let image {
                    Self.logger.debug("Loaded thumbnail for \(identifier): \(image.size)")
                    Task { @MainActor in
                        self.thumbnailImage = image
                    }
                }

                let isDegraded = (info?[PHImageResultIsDegradedKey] as? Bool) ?? false
                if !isDegraded {
                    continuation.resume()
                }
            }
        }
    }
}
