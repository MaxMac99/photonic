//
//  PhotonicMainView.swift
//  Photonic
//
//  Created by Max Vissing on 02.02.25.
//

import SwiftUI
import SwiftData

#if DEBUG
    import Inject
    import XcodebuildNvimPreview
#endif

struct PhotonicMainView: View {
    #if DEBUG
        @ObserveInjection var injection
    #endif

    @Environment(\.compositionRoot) private var compositionRoot
    @Environment(\.modelContext) private var modelContext

    var body: some View {
        TabView {
            Tab("Backup", systemImage: "arrow.up.circle") {
                if let root = compositionRoot {
                    BackupAlbumSelectionView(viewModel: root.makeBackupAlbumSelectionViewModel(modelContext: modelContext))
                } else {
                    Text("Loading...")
                }
            }
            Tab("Media", systemImage: "photo.stack") {
                MediaView()
            }
            Tab("Albums", systemImage: "rectangle.stack") {
                if let root = compositionRoot {
                    ImageSelectionView(viewModel: root.makeImageSelectionViewModel())
                } else {
                    Text("Loading...")
                }
            }
            Tab("Settings", systemImage: "gear") {
                if let root = compositionRoot {
                    SettingsView(viewModel: root.makeSettingsViewModel())
                } else {
                    Text("Loading...")
                }
            }
        }
        #if DEBUG
            .enableInjection()
            .setupNvimPreview {
                PhotonicMainView()
                .environment(\.apiClient, PreviewDependencies.createMockClient())
                .environment(\.compositionRoot, PreviewDependencies.createMockCompositionRoot())
            }
        #endif
    }
}

// MARK: - Previews

#Preview("Main View") {
    PhotonicMainView()
        .environment(\.apiClient, PreviewDependencies.createMockClient())
        .environment(\.compositionRoot, PreviewDependencies.createMockCompositionRoot())
}

#Preview("Main View - Dark Mode") {
    PhotonicMainView()
        .environment(\.apiClient, PreviewDependencies.createMockClient())
        .environment(\.compositionRoot, PreviewDependencies.createMockCompositionRoot())
        .preferredColorScheme(.dark)
}
