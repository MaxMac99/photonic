import ComposableArchitecture
import Foundation
@testable import Photonic

struct TestUploadFailure: Error, LocalizedError {
    var errorDescription: String? {
        "upload failed"
    }
}

struct FakeJobEntry {
    let job: UploadJob
    var status: UploadJob.Status
    var message: String?
}

/// In-memory fake of the durable queue with the same semantics as the
/// SwiftData adapter.
final class FakeQueue: @unchecked Sendable {
    private var jobs: [FakeJobEntry] = []

    init(jobs: [UploadJob]) {
        self.jobs = jobs.map { FakeJobEntry(job: $0, status: .pending, message: nil) }
    }

    func recoverStuckUploads() {
        for index in jobs.indices where jobs[index].status == .uploading {
            jobs[index].status = .pending
        }
    }

    func nextPending() -> UploadJob? {
        guard let index = jobs.firstIndex(where: { $0.status == .pending }) else { return nil }
        jobs[index].status = .uploading
        return jobs[index].job
    }

    func enqueue(_ newJobs: [UploadJob]) {
        jobs.append(
            contentsOf: newJobs.map { FakeJobEntry(job: $0, status: .pending, message: nil) }
        )
    }

    func enqueuedMediaIDs() -> [String] {
        jobs.map(\.job.mediaID)
    }

    func setStatus(_ status: UploadJob.Status, jobID: UUID, message: String?) {
        guard let index = jobs.firstIndex(where: { $0.job.id == jobID }) else { return }
        jobs[index].status = status
        jobs[index].message = message
    }

    func snapshot() -> BackupQueueSnapshot {
        var snapshot = BackupQueueSnapshot()
        for entry in jobs {
            switch entry.status {
            case .pending: snapshot.pending += 1
            case .uploading: snapshot.uploading += 1
            case .done: snapshot.done += 1
            case .failed: snapshot.failed += 1
            }
        }
        return snapshot
    }
}

/// Stalls the first upload attempt of each job (for pause/cancel tests).
final class FlakyUploader: @unchecked Sendable {
    private var stalled: Set<String> = []

    func handle(_ job: UploadJob) async throws {
        if !stalled.contains(job.mediaID) {
            stalled.insert(job.mediaID)
            try await Task.sleep(for: .seconds(60))
        }
    }
}

/// Deterministic photo-library fake.
final class FakePhotos: @unchecked Sendable {
    let albums: [PhotoAlbum]
    let pending: [PendingMedia]

    init(albums: [PhotoAlbum], pending: [PendingMedia]) {
        self.albums = albums
        self.pending = pending
    }
}

extension FakePhotos {
    func makeClient() -> PhotoLibraryClient {
        PhotoLibraryClient(
            requestAccess: { true },
            fetchAlbums: { [weak self] in await self?.albums ?? [] },
            pendingMedia: { [weak self] _ in await self?.pending ?? [] },
            loadData: { _ in Data() }
        )
    }
}

func makeBackupDeps(
    queue: FakeQueue,
    upload: @escaping @Sendable (UploadJob) async throws -> Void = { _ in }
) -> (BackupQueueClient, UploadClient) {
    (
        BackupQueueClient(
            recoverStuckUploads: { queue.recoverStuckUploads() },
            enqueue: { queue.enqueue($0) },
            nextPending: { queue.nextPending() },
            setStatus: { queue.setStatus($1, jobID: $0, message: $2) },
            observe: { .finished }
        ),
        UploadClient(upload: upload)
    )
}
