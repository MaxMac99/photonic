import Foundation
import PhotonicCore

/// One generated thumbnail ready for upload.
struct GeneratedThumbnail: Equatable, Sendable {
    let variant: ThumbnailVariant
    let data: Data
    let width: Int
    let height: Int
}

/// Payload for uploading a thumbnail variant as a preview item of an
/// already-created medium.
struct ThumbnailUpload: Equatable, Sendable {
    let variant: ThumbnailVariant
    let filename: String
    let width: Int
    let height: Int
    let data: Data
}
