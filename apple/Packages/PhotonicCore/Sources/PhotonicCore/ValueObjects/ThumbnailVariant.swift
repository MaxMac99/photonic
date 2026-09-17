import Foundation

/// Thumbnail size variants generated on-device and uploaded alongside the
/// original. Raw values match the spec's `ThumbnailVariantDto`.
public enum ThumbnailVariant: String, Sendable, Codable, CaseIterable {
    case tiny
    case small
    case large

    /// Maximum size (in pixels) of the longest edge for this variant.
    public var maxPixelSize: Int {
        switch self {
        case .tiny:
            200
        case .small:
            720
        case .large:
            1440
        }
    }
}
