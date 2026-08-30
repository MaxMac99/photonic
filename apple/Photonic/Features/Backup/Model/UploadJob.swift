import Foundation

/// A single durable unit of work: one media item to upload from one album.
struct UploadJob: Equatable, Identifiable, Sendable {
    enum Status: String, Sendable {
        case pending
        case uploading
        case done
        case failed
    }

    let id: UUID
    let albumID: String
    let albumName: String
    let mediaID: String

    init(id: UUID = UUID(), albumID: String, albumName: String, mediaID: String) {
        self.id = id
        self.albumID = albumID
        self.albumName = albumName
        self.mediaID = mediaID
    }
}

enum UploadOutcome: Equatable, Sendable {
    case success
    case failure(String?)
}

/// Point-in-time projection of the durable queue. The persisted store is the
/// source of truth (R12); this struct is what the feature renders.
struct BackupQueueSnapshot: Equatable, Sendable {
    var pending: Int = 0
    var uploading: Int = 0
    var done: Int = 0
    var failed: Int = 0

    static let empty = BackupQueueSnapshot()

    var total: Int {
        pending + uploading + done + failed
    }

    var processed: Int {
        done + failed
    }

    var hasPendingWork: Bool {
        pending > 0
    }
}

extension UploadJob {
    /// Placeholder payload until the photo-library adapter enqueues real
    /// selections (build-order step 5).
    static let samples: [UploadJob] = [
        UploadJob(albumID: "sample-album", albumName: "Camera Roll", mediaID: "sample-media-1"),
        UploadJob(albumID: "sample-album", albumName: "Camera Roll", mediaID: "sample-media-2"),
        UploadJob(albumID: "sample-album", albumName: "Camera Roll", mediaID: "sample-media-3")
    ]
}
