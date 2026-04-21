//
//  ImageSelectionViewModel.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import Foundation
import PhotosUI
import SwiftUI

@MainActor
public final class ImageSelectionViewModel: ObservableObject {
    private static let logger = LoggerFactory.logger(for: .ui)
    @Published var imageState: ImageState = .empty
    @Published var selectedItem: PhotosPickerItem? = nil

    private let photoLibraryAdapter: PhotoLibraryAdapter

    public init(
        photoLibraryAdapter: PhotoLibraryAdapter = PhotoLibraryAdapter()
    ) {
        self.photoLibraryAdapter = photoLibraryAdapter
    }

    public enum ImageState {
        case empty
        case loading
        case preview(Image)
        case success(Image, assetId: String)
        case error(Error)

        var canUpload: Bool {
            switch self {
            case .success:
                return true
            default:
                return false
            }
        }
    }

    public func handleImageSelection(_ item: PhotosPickerItem?) async {
        guard let item else { return }
        self.selectedItem = item

        guard let identifier = item.itemIdentifier else {
            Self.logger.error("No identifier found for selected item")
            imageState = .error(ImageSelectionError.noIdentifier)
            return
        }

        Self.logger.info("Selected image: \(identifier)")
        imageState = .loading

        // Load preview first
        await loadPreview(item: item)

        // Then load full image
        await loadFullImage(identifier: identifier)
    }

    private func loadPreview(item: PhotosPickerItem) async {
        do {
            if let data = try await item.loadTransferable(type: Data.self),
                let uiImage = UIImage(data: data)
            {
                Self.logger.debug("Preview loaded: \(uiImage.size)")
                let swiftImage = Image(uiImage: uiImage)
                if case .success = imageState {
                    // Don't override success state with preview
                } else {
                    imageState = .preview(swiftImage)
                }
            }
        } catch {
            Self.logger.error("Failed to load preview: \(error.localizedDescription)")
            imageState = .error(error)
        }
    }

    private func loadFullImage(identifier: String) async {
        do {
            let imageData = try await photoLibraryAdapter.loadPreviewImage(identifier: identifier)
            if let uiImage = UIImage(data: imageData) {
                Self.logger.debug("Full image loaded: \(uiImage.size)")
                imageState = .success(Image(uiImage: uiImage), assetId: identifier)
            }
        } catch {
            Self.logger.error("Failed to load full image: \(error.localizedDescription)")
            imageState = .error(error)
        }
    }

    public func uploadImage() async {
        guard case .success(_, let assetId) = imageState else {
            Self.logger.error("Cannot upload: invalid state")
            return
        }

        do {
            let (data, filename) = try await photoLibraryAdapter.fetchAssetData(identifier: assetId)
            Self.logger.info("Uploading \(filename): \(data.count) bytes")

            // TODO: Call upload use case when available
            // try await uploadMediaUseCase?.execute(data: data, filename: filename)

        } catch {
            Self.logger.error("Upload failed: \(error.localizedDescription)")
            imageState = .error(error)
        }
    }
}

enum ImageSelectionError: LocalizedError {
    case noIdentifier

    var errorDescription: String? {
        switch self {
        case .noIdentifier:
            return "No identifier found for selected image"
        }
    }
}

// Placeholder for upload use case
public protocol UploadMediaUseCase {
    func execute(data: Data, filename: String) async throws
}

