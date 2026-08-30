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
            VStack(spacing: 8) {
                Text("Ready to back up")
                    .font(.headline)
                Button("Start backup") {
                    store.send(.startTapped)
                }
                .buttonStyle(.borderedProminent)
                .disabled(!store.snapshot.hasPendingWork)
                #if DEBUG
                Button("Enqueue sample jobs") {
                    store.send(.enqueueSampleTapped)
                }
                .font(.footnote)
                #endif
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
