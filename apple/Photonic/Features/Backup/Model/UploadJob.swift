import Foundation
import PhotonicCore

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
    let mediaType: MediumType?
    let filename: String?
    let dateTaken: Date?

    init(
        id: UUID = UUID(),
        albumID: String,
        albumName: String,
        mediaID: String,
        mediaType: MediumType? = nil,
        filename: String? = nil,
        dateTaken: Date? = nil
    ) {
        self.id = id
        self.albumID = albumID
        self.albumName = albumName
        self.mediaID = mediaID
        self.mediaType = mediaType
        self.filename = filename
        self.dateTaken = dateTaken
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

/// A photo-library album offered for backup selection.
struct PhotoAlbum: Equatable, Identifiable, Sendable {
    let id: String
    let name: String
    let assetCount: Int
}

/// One asset discovered in a selected album, ready to become an upload job.
struct PendingMedia: Equatable, Identifiable, Sendable {
    let id: String
    let type: MediumType?
    let filename: String?
    let dateTaken: Date?
}
