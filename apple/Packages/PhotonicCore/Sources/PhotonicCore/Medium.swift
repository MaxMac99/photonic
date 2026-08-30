import Foundation

/// The kind of media a `Medium` represents. Raw values match the spec's
/// `MediumTypeDto` so mapping stays lossless.
public enum MediumType: String, Sendable, Codable, CaseIterable {
    case photo = "PHOTO"
    case video = "VIDEO"
    case livePhoto = "LIVE_PHOTO"
    case vector = "VECTOR"
    case sequence = "SEQUENCE"
    case gif = "GIF"
    case other = "OTHER"
}

/// One backed-up medium in the server's library (list-view shape).
public struct Medium: Hashable, Sendable, Identifiable, Codable {
    public let id: UUID
    public let type: MediumType
    public let albumID: UUID?
    public let takenAt: Date?
    public let primaryFilename: String?
    public let primaryFilesize: Int64?

    public init(
        id: UUID,
        type: MediumType,
        albumID: UUID?,
        takenAt: Date?,
        primaryFilename: String?,
        primaryFilesize: Int64?
    ) {
        self.id = id
        self.type = type
        self.albumID = albumID
        self.takenAt = takenAt
        self.primaryFilename = primaryFilename
        self.primaryFilesize = primaryFilesize
    }
}

/// Cursor-based paging position (R13: the library is paged, never held
/// whole). The API adapter derives it from a page's last item.
public struct MediaCursor: Hashable, Sendable, Codable {
    public let lastDate: Date?
    public let lastID: UUID

    public init(lastDate: Date?, lastID: UUID) {
        self.lastDate = lastDate
        self.lastID = lastID
    }
}

/// One fetched page plus the cursor to the next one (`nil` = end of library).
public struct MediaPage: Hashable, Sendable {
    public let media: [Medium]
    public let nextCursor: MediaCursor?

    public init(media: [Medium], nextCursor: MediaCursor?) {
        self.media = media
        self.nextCursor = nextCursor
    }
}
