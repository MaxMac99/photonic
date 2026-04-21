//
//  MediaView.swift
//  Photonic
//
//  Created by Max Vissing on 04.05.24.
//

import OpenAPIURLSession
import SwiftUI

struct MediaView: View {
    
    private static let logger = LoggerFactory.logger(for: .ui)

    private static let pageSize = 60

    @Environment(\.apiClient) private var client
    @State var media: [Components.Schemas.MediumResponse] = []
    @State var errorMessage: String?

    var body: some View {
        NavigationStack {
            if errorMessage != nil {
                Text("Failed to load media")
            } else if media.isEmpty {
                Text("There is no media in your library")
            } else {
                List(media) { medium in
                    MediumPreviewView(medium: medium)
                }
            }
        }
        .toolbarRole(.navigationStack)
        .task {
            await fetchMedia()
        }
    }

    func fetchMedia() async {
        do {
            errorMessage = nil
            media = try await client.get_all_media(.init()).ok.body.json
            Self.logger.info("Received media: \(media.count)")
        } catch {
            Self.logger.error("Error fetching media", error: error)
            errorMessage = error.localizedDescription
        }
    }
}

extension Components.Schemas.MediumResponse: Identifiable {
}

#Preview {
    MediaView()
        .environment(\.apiClient, MockAPI())
}
