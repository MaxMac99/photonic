import ComposableArchitecture
import Dependencies
import Foundation

/// Orchestrates the backup pipeline. The durable queue is the source of truth
/// (R12); this reducer is its projection and drives the processing loop.
@Reducer
struct BackupFeature {
    @Dependency(BackupQueueClient.self) private var queue
    @Dependency(UploadClient.self) private var uploads

    enum Phase: Equatable {
        case idle
        case processing
        case paused
        case completed
    }

    private enum CancelID {
        case processing
    }

    @ObservableState
    struct State: Equatable {
        var phase: Phase = .idle
        var snapshot: BackupQueueSnapshot = .empty
        var currentJob: UploadJob?
        var lastErrorMessage: String?
    }

    enum Action: Equatable {
        case onAppear
        case enqueueSampleTapped
        case startTapped
        case pauseTapped
        case resumeTapped
        case cancelTapped
        case queueDidChange(BackupQueueSnapshot)
        case jobDidStart(UploadJob)
        case jobDidFinish(UUID, UploadOutcome)
        case processingFinished
    }

    var body: some ReducerOf<Self> {
        Reduce { state, action in
            switch action {
            case .onAppear:
                return .run { send in
                    try await queue.recoverStuckUploads()
                    let snapshots = await queue.observe()
                    for await snapshot in snapshots {
                        await send(.queueDidChange(snapshot))
                    }
                }

            case .enqueueSampleTapped:
                return .run { _ in
                    try await queue.enqueue(UploadJob.samples)
                }

            case .startTapped, .resumeTapped:
                guard state.snapshot.hasPendingWork else { return .none }
                state.phase = .processing
                state.lastErrorMessage = nil
                return .run { send in
                    while let job = try await queue.nextPending() {
                        try Task.checkCancellation()
                        await send(.jobDidStart(job))
                        do {
                            try await uploads.upload(job)
                            try await queue.setStatus(job.id, .done, nil)
                            await send(.jobDidFinish(job.id, .success))
                        } catch is CancellationError {
                            return
                        } catch {
                            try await queue.setStatus(
                                job.id,
                                .failed,
                                error.localizedDescription
                            )
                            await send(.jobDidFinish(job.id, .failure(error.localizedDescription)))
                        }
                    }
                    await send(.processingFinished)
                }
                .cancellable(id: CancelID.processing, cancelInFlight: true)

            case .pauseTapped:
                state.phase = .paused
                guard let job = state.currentJob else {
                    return .cancel(id: CancelID.processing)
                }
                state.currentJob = nil
                state.snapshot.uploading -= 1
                state.snapshot.pending += 1
                return .concatenate(
                    .cancel(id: CancelID.processing),
                    .run { _ in try await queue.setStatus(job.id, .pending, nil) }
                )

            case .cancelTapped:
                state.phase = .idle
                var effects: [Effect<Action>] = [.cancel(id: CancelID.processing)]
                if let job = state.currentJob {
                    state.currentJob = nil
                    state.snapshot.uploading -= 1
                    state.snapshot.pending += 1
                    effects.append(.run { _ in try await queue.setStatus(job.id, .pending, nil) })
                }
                return .merge(effects)

            case let .queueDidChange(snapshot):
                state.snapshot = snapshot
                return .none

            case let .jobDidStart(job):
                state.currentJob = job
                state.snapshot.pending -= 1
                state.snapshot.uploading += 1
                return .none

            case let .jobDidFinish(_, outcome):
                state.currentJob = nil
                state.snapshot.uploading -= 1
                switch outcome {
                case .success:
                    state.snapshot.done += 1
                case let .failure(message):
                    state.snapshot.failed += 1
                    state.lastErrorMessage = message
                }
                return .none

            case .processingFinished:
                if state.phase == .processing {
                    state.phase = .completed
                }
                return .none
            }
        }
    }
}
