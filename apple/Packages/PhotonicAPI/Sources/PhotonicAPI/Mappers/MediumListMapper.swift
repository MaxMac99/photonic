import Foundation
import PhotonicCore

/// Maps generated list payloads to Core media models, deriving the next
/// paging cursor from the page's last item (R13).
public enum MediumListMapper {
    public static func map(
        _ payload: [Components.Schemas.MediumListResponse],
        pageSize: Int
    ) throws -> MediaPage {
        let media = try payload.map(mapMedium)

        guard media.count == pageSize, let last = media.last else {
            return MediaPage(media: media, nextCursor: nil)
        }
        return MediaPage(
            media: media,
            nextCursor: MediaCursor(lastDate: last.takenAt, lastID: last.id)
        )
    }

    private static func mapMedium(_ payload: Components.Schemas.MediumListResponse) throws -> Medium {
        let primaryItem = payload.items.first(where: \.is_primary) ?? payload.items.first
        guard
            let id = UUID(uuidString: payload.id)
        else {
            throw APIMappingError.invalidPayload("medium list carries an invalid medium id")
        }
        return Medium(
            id: id,
            type: MediumType(rawValue: payload.medium_type.rawValue) ?? .other,
            albumID: payload.album_id.flatMap(UUID.init(uuidString:)),
            takenAt: payload.taken_at,
            primaryFilename: primaryItem?.filename,
            primaryFilesize: primaryItem?.filesize
        )
    }
}
