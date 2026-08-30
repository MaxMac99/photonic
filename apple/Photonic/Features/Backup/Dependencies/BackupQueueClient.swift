import ComposableArchitecture
import Dependencies
import Foundation

/// Typed interface to the durable upload queue (R12). The persisted store is
/// the source of truth; this client is the only way feature code touches it.
@DependencyClient
struct BackupQueueClient: Sendable {
    var recoverStuckUploads: @Sendable () async throws -> Void
    var enqueue: @Sendable ([UploadJob]) async throws -> Void
    /// Claims the next pending job (atomically marks it uploading) or returns
    /// nil when the queue is drained.
    var nextPending: @Sendable () async throws -> UploadJob?
    var setStatus: @Sendable (UploadJob.ID, UploadJob.Status, String?) async throws -> Void
    /// Yields the current snapshot immediately, then on every queue mutation.
    var observe: @Sendable () async -> AsyncStream<BackupQueueSnapshot> = { .finished }
}

extension DependencyValues {
    var backupQueueClient: BackupQueueClient {
        get { self[BackupQueueClient.self] }
        set { self[BackupQueueClient.self] = newValue }
    }
}
