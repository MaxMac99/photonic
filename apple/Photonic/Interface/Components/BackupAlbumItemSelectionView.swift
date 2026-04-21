//
//  BackupAlbumItemSelectionView.swift
//  Photonic
//
//  Created by Max Vissing on 02.03.25.
//

import Photos
import SwiftUI

public struct BackupAlbumItemSelectionView: View {

    @StateObject private var viewModel: BackupAlbumItemViewModel
    let selection: BackupAlbumSelectionEntity?
    let onSelectionChange: (AlbumSelectionType?) -> Void
    
    private let height: CGFloat = 124
    
    public init(
        collection: PHAssetCollection,
        selection: BackupAlbumSelectionEntity?,
        onSelectionChange: @escaping (AlbumSelectionType?) -> Void
    ) {
        self._viewModel = StateObject(wrappedValue: BackupAlbumItemViewModel(collection: collection))
        self.selection = selection
        self.onSelectionChange = onSelectionChange
    }

    public var body: some View {
        HStack(alignment: .center) {
            HStack(alignment: .center, spacing: 20) {
                Group {
                    if let selection = selection {
                        switch selection.selectionType {
                        case .included:
                            Image(systemName: "checkmark.circle.fill")
                                .symbolRenderingMode(.multicolor)
                        case .excluded:
                            Image(systemName: "minus.circle.fill")
                                .symbolRenderingMode(.multicolor)
                        }
                    } else {
                        Image(systemName: "circle")
                    }
                }
                VStack(alignment: .leading) {
                    Text(viewModel.title)
                        .bold()
                    Text("\(viewModel.assetCount)")
                }
            }
            .padding()

            Spacer()

            if let image = viewModel.thumbnailImage {
                Image(uiImage: image)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
                    .frame(
                        width: height,
                        height: height
                    )
                    .clipped()
            } else if viewModel.isLoadingImage {
                ProgressView()
                    .padding()
            } else {
                EmptyView()
            }
        }
        .task {
            await viewModel.loadThumbnail()
        }
        .contentShape(Rectangle())
        .onTapGesture {
            if selection != nil {
                // Remove selection
                onSelectionChange(nil)
            } else {
                // Add as included
                onSelectionChange(.included)
            }
        }
        .onTapGesture(count: 2) {
            if let selection = selection, selection.selectionType == .excluded {
                // Remove exclusion
                onSelectionChange(nil)
            } else {
                // Add as excluded
                onSelectionChange(.excluded)
            }
        }
    }

}

#Preview {
    VStack {
        BackupAlbumItemSelectionView(
            collection: PHAssetCollection(),
            selection: nil,
            onSelectionChange: { _ in }
        )
        BackupAlbumItemSelectionView(
            collection: PHAssetCollection(),
            selection: BackupAlbumSelectionEntity(
                albumIdentifier: "test",
                albumName: "Test Album",
                selectionType: .included
            ),
            onSelectionChange: { _ in }
        )
        BackupAlbumItemSelectionView(
            collection: PHAssetCollection(),
            selection: BackupAlbumSelectionEntity(
                albumIdentifier: "test",
                albumName: "Test Album",
                selectionType: .excluded
            ),
            onSelectionChange: { _ in }
        )
    }
}
