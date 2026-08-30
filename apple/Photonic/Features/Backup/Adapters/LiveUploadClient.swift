import Dependencies
import Foundation

enum UploadError: Error, Sendable {
    case mediaSourceUnavailable
}

/// Live upload implementation. The durable queue plus relaunch recovery
/// (R12) already carry the correctness guarantees; real byte transfer lands
/// with the photo-library adapter (build-order step 5), which feeds
/// `PhotonicAPI.createMedium` — including the swap to a background
/// `URLSession` for actual transfers.
extension UploadClient: DependencyKey {
    static var liveValue: UploadClient {
        UploadClient(
            upload: { _ in
                throw UploadError.mediaSourceUnavailable
            }
        )
    }
}
