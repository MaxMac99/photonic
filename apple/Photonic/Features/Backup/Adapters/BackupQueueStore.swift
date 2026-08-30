import Dependencies
import Foundation
import PhotonicCore
import SwiftData

/// Persisted representation of one upload job. SwiftData is confined to this
/// adapter (R3); feature code only sees `UploadJob`/`BackupQueueSnapshot`.
@Model
final class PersistedUploadJob {
    @Attribute(.unique) var jobID: UUID
    var albumID: String
    var albumName: String
    var mediaID: String
    var mediaTypeRawValue: String?
    var filename: String?
    var dateTaken: Date?
    var statusRawValue: String
    var attempts: Int
    var errorMessage: String?
    var updatedAt: Date

    init(
        jobID: UUID,
        albumID: String,
        albumName: String,
        mediaID: String,
        mediaTypeRawValue: String? = nil,
        filename: String? = nil,
        dateTaken: Date? = nil,
        statusRawValue: String,
        attempts: Int = 0,
        errorMessage: String? = nil,
        updatedAt: Date
    ) {
        self.jobID = jobID
        self.albumID = albumID
        self.albumName = albumName
        self.mediaID = mediaID
        self.mediaTypeRawValue = mediaTypeRawValue
        self.filename = filename
        self.dateTaken = dateTaken
        self.statusRawValue = statusRawValue
        self.attempts = attempts
        self.errorMessage = errorMessage
        self.updatedAt = updatedAt
    }
}

/// Actor-backed access to the durable queue. Every mutation notifies the
/// observers so the app reflects durable truth (R12).
@ModelActor
actor BackupQueueStore {
    private var continuations: [UUID: AsyncStream<BackupQueueSnapshot>.Continuation] = [:]

    static func shared(inMemory: Bool = false) -> BackupQueueStore {
        let configuration = ModelConfiguration(isStoredInMemoryOnly: inMemory)
        do {
            let container = try ModelContainer(for: PersistedUploadJob.self, configurations: configuration)
            return BackupQueueStore(modelContainer: container)
        } catch {
            fatalError("Failed to create the backup queue store: \(error)")
        }
    }

    func enqueue(_ jobs: [UploadJob]) throws {
        let now = Date()
        for job in jobs {
            modelContext.insert(
                PersistedUploadJob(
                    jobID: job.id,
                    albumID: job.albumID,
                    albumName: job.albumName,
                    mediaID: job.mediaID,
                    mediaTypeRawValue: job.mediaType?.rawValue,
                    filename: job.filename,
                    dateTaken: job.dateTaken,
                    statusRawValue: UploadJob.Status.pending.rawValue,
                    updatedAt: now
                )
            )
        }
        try modelContext.save()
        notify()
    }

    func nextPending() throws -> UploadJob? {
        let pendingStatus = UploadJob.Status.pending.rawValue
        var descriptor = FetchDescriptor<PersistedUploadJob>(
            predicate: #Predicate { $0.statusRawValue == pendingStatus },
            sortBy: [SortDescriptor(\.updatedAt)]
        )
        descriptor.fetchLimit = 1

        guard let persisted = try modelContext.fetch(descriptor).first else { return nil }
        persisted.statusRawValue = UploadJob.Status.uploading.rawValue
        persisted.attempts += 1
        persisted.updatedAt = Date()
        try modelContext.save()
        notify()
        return persisted.asUploadJob
    }

    func setStatus(_ status: UploadJob.Status, for jobID: UUID, errorMessage: String?) throws {
        guard let persisted = fetchJob(id: jobID) else { return }
        persisted.statusRawValue = status.rawValue
        persisted.errorMessage = errorMessage
        persisted.updatedAt = Date()
        try modelContext.save()
        notify()
    }

    /// Crash/relaunch recovery (R12): jobs stuck in `uploading` become
    /// pending again, because the durable queue is the source of truth.
    func recoverStuckUploads() throws {
        let uploadingStatus = UploadJob.Status.uploading.rawValue
        let descriptor = FetchDescriptor<PersistedUploadJob>(
            predicate: #Predicate { $0.statusRawValue == uploadingStatus }
        )
        let stuck = try modelContext.fetch(descriptor)
        guard !stuck.isEmpty else { return }
        for job in stuck {
            job.statusRawValue = UploadJob.Status.pending.rawValue
            job.updatedAt = Date()
        }
        try modelContext.save()
        notify()
    }

    func observe() -> AsyncStream<BackupQueueSnapshot> {
        let id = UUID()
        return AsyncStream { continuation in
            continuation.yield(currentSnapshot())
            self.continuations[id] = continuation
            continuation.onTermination = { [weak self] _ in
                Task { await self?.removeContinuation(id) }
            }
        }
    }

    // MARK: - Private

    private func fetchJob(id: UUID) -> PersistedUploadJob? {
        var descriptor = FetchDescriptor<PersistedUploadJob>(
            predicate: #Predicate { $0.jobID == id }
        )
        descriptor.fetchLimit = 1
        return try? modelContext.fetch(descriptor).first
    }

    private func currentSnapshot() -> BackupQueueSnapshot {
        var snapshot = BackupQueueSnapshot()
        guard let all = try? modelContext.fetch(FetchDescriptor<PersistedUploadJob>()) else {
            return snapshot
        }
        for job in all {
            switch job.statusRawValue {
            case UploadJob.Status.pending.rawValue: snapshot.pending += 1
            case UploadJob.Status.uploading.rawValue: snapshot.uploading += 1
            case UploadJob.Status.done.rawValue: snapshot.done += 1
            case UploadJob.Status.failed.rawValue: snapshot.failed += 1
            default: break
            }
        }
        return snapshot
    }

    private func notify() {
        let snapshot = currentSnapshot()
        for continuation in continuations.values {
            continuation.yield(snapshot)
        }
    }

    private func removeContinuation(_ id: UUID) {
        continuations[id] = nil
    }
}

private extension PersistedUploadJob {
    var asUploadJob: UploadJob {
        UploadJob(
            id: jobID,
            albumID: albumID,
            albumName: albumName,
            mediaID: mediaID,
            mediaType: mediaTypeRawValue.flatMap(MediumType.init(rawValue:)),
            filename: filename,
            dateTaken: dateTaken
        )
    }
}

// MARK: - Dependency registration (R10)

extension BackupQueueClient: DependencyKey {
    static var liveValue: BackupQueueClient {
        makeClient(store: .shared())
    }

    /// In-memory store for previews.
    static var inMemoryValue: BackupQueueClient {
        makeClient(store: .shared(inMemory: true))
    }

    private static func makeClient(store: BackupQueueStore) -> BackupQueueClient {
        BackupQueueClient(
            recoverStuckUploads: { try await store.recoverStuckUploads() },
            enqueue: { try await store.enqueue($0) },
            nextPending: { try await store.nextPending() },
            setStatus: { try await store.setStatus($1, for: $0, errorMessage: $2) },
            observe: { await store.observe() }
        )
    }
}
