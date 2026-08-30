import ComposableArchitecture
import SwiftUI

struct BackupView: View {
    let store: StoreOf<BackupFeature>

    var body: some View {
        VStack(spacing: 16) {
            phaseView
            countsView
            if let message = store.lastErrorMessage {
                Text(message)
                    .font(.footnote)
                    .foregroundStyle(.red)
                    .multilineTextAlignment(.center)
            }
        }
        .padding()
        .onAppear {
            store.send(.onAppear)
        }
    }

    @ViewBuilder
    private var phaseView: some View {
        switch store.phase {
        case .idle:
            VStack(spacing: 12) {
                if store.albums.isEmpty {
                    Button("Choose albums to back up") {
                        store.send(.chooseAlbumsTapped)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(store.isLoadingAlbums)
                    if store.isLoadingAlbums {
                        ProgressView()
                    }
                } else {
                    albumList
                }
                if store.snapshot.hasPendingWork {
                    Button("Start backup") {
                        store.send(.startTapped)
                    }
                    .buttonStyle(.borderedProminent)
                }
            }
        case .processing:
            VStack(spacing: 12) {
                ProgressView(value: store.snapshot.progressFraction)
                if let job = store.currentJob {
                    Text("Uploading from \(job.albumName)")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
                HStack {
                    Button("Pause") { store.send(.pauseTapped) }
                    Button("Cancel", role: .destructive) { store.send(.cancelTapped) }
                }
            }
        case .paused:
            VStack(spacing: 12) {
                Text("Paused")
                    .font(.headline)
                HStack {
                    Button("Resume") { store.send(.resumeTapped) }
                        .buttonStyle(.borderedProminent)
                    Button("Cancel", role: .destructive) { store.send(.cancelTapped) }
                }
            }
        case .completed:
            VStack(spacing: 12) {
                Text("Backup finished")
                    .font(.headline)
                if store.snapshot.failed > 0 {
                    Text("\(store.snapshot.failed) item(s) failed")
                        .foregroundStyle(.red)
                }
                Button("Close") { store.send(.cancelTapped) }
            }
        }
    }

    private var albumList: some View {
        VStack(spacing: 8) {
            Text("Choose albums")
                .font(.headline)
            ForEach(store.albums) { album in
                Button {
                    store.send(.albumToggled(album.id))
                } label: {
                    HStack {
                        Image(
                            systemName: store.selectedAlbumIDs.contains(album.id)
                                ? "checkmark.circle.fill"
                                : "circle"
                        )
                        .foregroundStyle(Color.accentColor)
                        Text(album.name)
                        Spacer()
                        Text("\(album.assetCount)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                .buttonStyle(.plain)
            }
            Button(
                store.isPreparingSelection ? "Preparing…" :
                    "Back up \(store.selectedAlbumIDs.count) album(s)"
            ) {
                store.send(.enqueueSelectionTapped)
            }
            .buttonStyle(.borderedProminent)
            .disabled(store.selectedAlbumIDs.isEmpty || store.isPreparingSelection)
        }
    }

    private var countsView: some View {
        let snapshot = store.snapshot
        return Grid {
            GridRow {
                Text("Pending").foregroundStyle(.secondary)
                Text("\(snapshot.pending)").monospacedDigit()
            }
            GridRow {
                Text("Done").foregroundStyle(.secondary)
                Text("\(snapshot.done)").monospacedDigit()
            }
            GridRow {
                Text("Failed").foregroundStyle(.secondary)
                Text("\(snapshot.failed)").monospacedDigit()
            }
        }
    }
}

private extension BackupQueueSnapshot {
    var progressFraction: Double {
        guard total > 0 else { return 0 }
        return Double(processed) / Double(total)
    }
}

#Preview {
    BackupView(
        store: Store(initialState: BackupFeature.State()) {
            BackupFeature()
        } withDependencies: {
            $0.backupQueueClient = .inMemoryValue
        }
    )
}
