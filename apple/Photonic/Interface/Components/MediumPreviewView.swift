//
//  PhotonicImage.swift
//  Photonic
//
//  Created by Max Vissing on 26.01.25.
//

import OpenAPIURLSession
import SwiftUI

struct MediumPreviewView: View {

    private static let logger = LoggerFactory.logger(for: .ui)

    @Environment(\.apiClient) var client

    let medium: Components.Schemas.MediumResponse
    @State var image: Data?

    var body: some View {
        if let image = currentImage {
            Image(uiImage: image)
        } else {
            EmptyView()
                .task {
                    await startFetchingImages()
                }
        }
    }

    var currentImage: UIImage? {
        if let image {
            return UIImage(data: image)
        }
        return nil
    }

    func startFetchingImages() async {
        Self.logger.debug("Fetching preview for medium: \(medium.id)")

        do {
            let response = try await client.get_medium_preview(
                .init(path: .init(medium_id: medium.id))
            ).ok.body.any
            let imageData = try await Data(collecting: response, upTo: Int.max)
            image = imageData
            Self.logger.debug(
                "Preview loaded for medium: \(medium.id), size: \(imageData.count) bytes")
        } catch {
            Self.logger.error("Failed to fetch preview for medium: \(medium.id)", error: error)
        }
    }
}

//#Preview {
//    PhotonicImage()
//}
