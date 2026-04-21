//
//  ImageSelectionView.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import PhotosUI
import SwiftUI

public struct ImageSelectionView: View {
    @ObservedObject var viewModel: ImageSelectionViewModel

    public var body: some View {
        VStack {
            Spacer()
                .frame(height: 30)

            PhotosPicker(
                selection: $viewModel.selectedItem,
                matching: .images,
                photoLibrary: .shared()
            ) {
                Text("Select Image")
                    .padding(
                        EdgeInsets(
                            top: 10, leading: 20, bottom: 10, trailing: 20
                        )
                    )
                    .foregroundStyle(.primary)
                    .background(
                        .ultraThinMaterial,
                        in: RoundedRectangle(
                            cornerRadius: 25,
                            style: .continuous
                        )
                    )
            }
            .buttonStyle(.borderless)

            Spacer()

            switch viewModel.imageState {
            case .empty:
                EmptyView()
            case .loading:
                ProgressView()
            case .preview(let image), .success(let image, _):
                image
                    .resizable()
                    .aspectRatio(contentMode: .fit)
            case .error(let error):
                VStack(spacing: 10) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .font(.system(size: 40))
                        .foregroundColor(.red)
                    Text(error.localizedDescription)
                        .foregroundColor(.secondary)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal)
                }
            }

            Spacer()

            Button("Upload") {
                Task {
                    await viewModel.uploadImage()
                }
            }
            .buttonStyle(.bordered)
            .controlSize(.extraLarge)
            .background(.thickMaterial, in: Capsule())
            .disabled(!viewModel.imageState.canUpload)

            Spacer()
                .frame(height: 30)
        }
        .onChange(of: viewModel.selectedItem) { _, newValue in
            Task {
                await viewModel.handleImageSelection(newValue)
            }
        }
    }
}

#Preview {
    ImageSelectionView(viewModel: ImageSelectionViewModel())
}
