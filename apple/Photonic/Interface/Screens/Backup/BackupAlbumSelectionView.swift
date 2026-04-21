//
//  BackupAlbumSelectionView.swift
//  Photonic
//
//  Created by Max Vissing on 14.02.25.
//

import PhotosUI
import SwiftUI

public struct BackupAlbumSelectionView: View {

    @ObservedObject var viewModel: BackupAlbumSelectionViewModel

    public var body: some View {
        List {
            if !viewModel.selections.isEmpty {
                FlowLayout {
                    ForEach(viewModel.selections) { selection in
                        HStack {
                            Text(selection.albumName)
                            Image(systemName: "xmark")
                        }
                        .padding(.vertical, 8)
                        .padding(.horizontal)
                        .background(
                            Capsule()
                                .fill(tagColor(for: selection.selectionType).opacity(0.3))
                                .stroke(tagColor(for: selection.selectionType), lineWidth: 1)
                        )
                        .onTapGesture {
                            Task {
                                await viewModel.removeSelection(selection)
                            }
                        }
                        .padding(4)
                    }
                }
            }
            ForEach(viewModel.collections, id: \.localIdentifier) { collection in
                BackupAlbumItemSelectionView(
                    collection: collection,
                    selection: viewModel.selection(for: collection),
                    onSelectionChange: { newType in
                        Task {
                            await viewModel.updateSelection(
                                for: collection,
                                type: newType
                            )
                        }
                    }
                )
                .listRowInsets(EdgeInsets())
            }
        }
        .listStyle(.inset)
        .task {
            await viewModel.loadData()
        }
        .navigationTitle("Select Albums")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("Start Backup") {
                    Task {
                        await viewModel.startBackup()
                    }
                }
                .disabled(!viewModel.hasSelectionsForBackup || viewModel.isBackupInProgress)
            }
        }
        .sheet(isPresented: $viewModel.showBackupSheet) {
            BackupProgressView(viewModel: viewModel)
        }
        .onChange(of: viewModel.isBackupInProgress) { isInProgress in
            if isInProgress {
                viewModel.showBackupSheet = true
            }
        }
        .alert("Error", isPresented: $viewModel.showError) {
            Button("OK") { }
        } message: {
            Text(viewModel.errorMessage)
        }
    }

    private func tagColor(for type: AlbumSelectionType) -> Color {
        switch type {
        case .included: return .green
        case .excluded: return .red
        }
    }
}

#Preview {
    BackupAlbumSelectionView(
        viewModel: BackupAlbumSelectionViewModel(
            backupSelectionRepository: MockBackupSelectionRepository()
        )
    )
}

// Mock for preview
private class MockBackupSelectionRepository: BackupSelectionRepository {
    func fetchSelections() async throws -> [BackupAlbumSelectionEntity] { [] }
    func saveSelection(_ selection: BackupAlbumSelectionEntity) async throws {}
    func deleteSelection(withId id: String) async throws {}
    func findSelection(byAlbumId albumId: String) async throws -> BackupAlbumSelectionEntity? {
        nil
    }
    func clearAllSelections() async throws {}
}
