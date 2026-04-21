//
//  PhotoLibraryAdapter.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import Foundation
import Photos
import OSLog

public final class PhotoLibraryAdapter {
    private static let logger = Logger(
        subsystem: Bundle.main.bundleIdentifier!,
        category: String(describing: PhotoLibraryAdapter.self)
    )
    
    public init() {}
    
    public func fetchAssetData(identifier: String) async throws -> (data: Data, filename: String) {
        let assets = PHAsset.fetchAssets(withLocalIdentifiers: [identifier], options: nil)
        
        guard let asset = assets.firstObject else {
            Self.logger.error("No asset found for identifier \(identifier)")
            throw PhotoLibraryError.assetNotFound(identifier)
        }
        
        guard let resource = PHAssetResource.assetResources(for: asset).first else {
            Self.logger.error("No resource found for asset \(identifier)")
            throw PhotoLibraryError.resourceNotFound(identifier)
        }
        
        let data: Data = try await withCheckedThrowingContinuation { continuation in
            let options = PHAssetResourceRequestOptions()
            options.isNetworkAccessAllowed = true
            
            let buffer = NSMutableData()
            PHAssetResourceManager.default().requestData(
                for: resource, options: options,
                dataReceivedHandler: { data in
                    buffer.append(data)
                },
                completionHandler: { error in
                    if let error {
                        Self.logger.error("Error loading data: \(error.localizedDescription)")
                        continuation.resume(throwing: PhotoLibraryError.dataLoadFailed(error))
                    } else {
                        Self.logger.debug("Data loaded successfully for asset \(identifier)")
                        continuation.resume(returning: buffer as Data)
                    }
                }
            )
        }
        
        Self.logger.info("Loaded asset \(identifier): \(resource.originalFilename), \(data.count) bytes")
        return (data, resource.originalFilename)
    }
    
    func getData(for item: MediaItem) async throws -> Data {
        let (data, _) = try await fetchAssetData(identifier: item.id)
        return data
    }
    
    public func loadPreviewImage(identifier: String) async throws -> Data {
        let results = PHAsset.fetchAssets(withLocalIdentifiers: [identifier], options: nil)
        guard let asset = results.firstObject else {
            throw PhotoLibraryError.assetNotFound(identifier)
        }
        
        return try await withCheckedThrowingContinuation { continuation in
            let options = PHImageRequestOptions()
            options.isNetworkAccessAllowed = true
            options.version = .current
            options.deliveryMode = .highQualityFormat
            options.resizeMode = .none
            
            PHImageManager.default().requestImageDataAndOrientation(
                for: asset, 
                options: options
            ) { data, _, _, info in
                let degraded = info?[PHImageResultIsDegradedKey] as? Bool
                if !(degraded ?? false), let data {
                    continuation.resume(returning: data)
                } else {
                    continuation.resume(throwing: PhotoLibraryError.previewLoadFailed)
                }
            }
        }
    }
}

public enum PhotoLibraryError: LocalizedError {
    case assetNotFound(String)
    case resourceNotFound(String)
    case dataLoadFailed(Error)
    case previewLoadFailed
    
    public var errorDescription: String? {
        switch self {
        case .assetNotFound(let id):
            return "Asset not found: \(id)"
        case .resourceNotFound(let id):
            return "Resource not found for asset: \(id)"
        case .dataLoadFailed(let error):
            return "Failed to load data: \(error.localizedDescription)"
        case .previewLoadFailed:
            return "Failed to load preview image"
        }
    }
}