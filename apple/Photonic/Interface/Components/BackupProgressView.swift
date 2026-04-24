//
//  BackupProgressView.swift
//  Photonic
//
//  Interface Layer - Backup Progress Display
//

import SwiftUI

public struct BackupProgressView: View {
    @ObservedObject var viewModel: BackupAlbumSelectionViewModel

    public var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                // Status Header
                VStack(spacing: 8) {
                    Text(viewModel.statusMessage)
                        .font(.headline)
                        .multilineTextAlignment(.center)

                    if viewModel.isBackupInProgress {
                        Text(viewModel.progressText)
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                }
                .padding()

                // Progress Bar
                if let progress = viewModel.backupProgress {
                    VStack(spacing: 12) {
                        ProgressView(
                            value: Double(progress.processedItems),
                            total: Double(progress.totalItems)
                        )
                        .progressViewStyle(.linear)

                        HStack {
                            Text("\(Int(progress.progressPercentage))%")
                                .font(.caption)
                                .foregroundColor(.secondary)

                            Spacer()

                            Text("\(progress.processedItems) / \(progress.totalItems)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(.horizontal)

                    // Current Item
                    if let currentItem = progress.currentItem, progress.status == .uploading {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Currently uploading:")
                                .font(.caption)
                                .foregroundColor(.secondary)

                            Text(currentItem.id)
                                .font(.caption)
                                .foregroundColor(.primary)
                                .lineLimit(1)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal)
                    }

                    // Errors
                    if !progress.errors.isEmpty {
                        VStack(alignment: .leading, spacing: 8) {
                            HStack {
                                Image(systemName: "exclamationmark.triangle.fill")
                                    .foregroundColor(.orange)
                                Text("Errors (\(progress.errors.count))")
                                    .font(.subheadline)
                                    .foregroundColor(.orange)
                            }

                            ScrollView {
                                LazyVStack(alignment: .leading, spacing: 4) {
                                    ForEach(progress.errors.suffix(5), id: \.mediaId) { error in
                                        VStack(alignment: .leading, spacing: 2) {
                                            Text("Item: \(error.mediaId)")
                                                .font(.caption2)
                                                .foregroundColor(.secondary)
                                            Text(error.message)
                                                .font(.caption)
                                                .foregroundColor(.orange)
                                        }
                                        .padding(.vertical, 2)
                                    }
                                }
                            }
                            .frame(maxHeight: 100)
                        }
                        .padding(.horizontal)
                    }
                }

                Spacer()

                // Control Buttons
                HStack(spacing: 16) {
                    if viewModel.isBackupInProgress {
                        if viewModel.backupProgress?.status == .paused {
                            Button("Resume") {
                                Task {
                                    await viewModel.resumeBackup()
                                }
                            }
                            .buttonStyle(.borderedProminent)
                        } else {
                            Button("Pause") {
                                Task {
                                    await viewModel.pauseBackup()
                                }
                            }
                            .buttonStyle(.bordered)
                        }

                        Button("Cancel") {
                            Task {
                                await viewModel.cancelBackup()
                            }
                        }
                        .buttonStyle(.bordered)
                        .foregroundColor(.red)
                    }
                }
                .padding(.horizontal)
            }
            .navigationTitle("Backup Progress")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    if !viewModel.isBackupInProgress {
                        Button("Done") {
                            viewModel.showBackupSheet = false
                        }
                    }
                }
            }
        }
    }
}

#Preview {
    BackupProgressView(
        viewModel: BackupAlbumSelectionViewModel(
            backupSelectionRepository: MockBackupSelectionRepository()
        )
    )
}

/// Mock for preview
private class MockBackupSelectionRepository: BackupSelectionRepository {
    func fetchSelections() async throws -> [BackupAlbumSelectionEntity] {
        []
    }

    func saveSelection(_ selection: BackupAlbumSelectionEntity) async throws {}
    func deleteSelection(withId id: String) async throws {}
    func findSelection(byAlbumId albumId: String) async throws -> BackupAlbumSelectionEntity? {
        nil
    }

    func clearAllSelections() async throws {}
}
