import Foundation
import PhotonicCore
import Testing
@testable import PhotonicAPI

struct MediumListMapperTests {
    private static func payload(
        id: String,
        takenAt: String? = "2024-06-01T12:00:00Z",
        albumID: String? = nil,
        primary: Bool = true
    ) -> Components.Schemas.MediumListResponse {
        Components.Schemas.MediumListResponse(
            album_id: albumID,
            camera_make: nil,
            camera_model: nil,
            id: id,
            items: [
                Components.Schemas.MediumItemResponse(
                    filename: "IMG_0001.HEIC",
                    filesize: 2_400_000,
                    height: nil,
                    id: "item-1",
                    is_primary: primary,
                    medium_item_type: nil,
                    mime: "image/heic",
                    width: nil
                )
            ],
            medium_type: .photo,
            taken_at: takenAt
        )
    }

    @Test
    func mapsMediaAndDerivesCursorForFullPage() throws {
        let page = try MediumListMapper.map(
            [payload(id: "11111111-1111-1111-1111-111111111111")],
            pageSize: 1
        )

        #expect(page.media.count == 1)
        #expect(page.media[0].type == .photo)
        #expect(page.media[0].primaryFilename == "IMG_0001.HEIC")
        let cursor = try #require(page.nextCursor)
        #expect(cursor.lastID == UUID(uuidString: "11111111-1111-1111-1111-111111111111"))
        #expect(cursor.lastDate != nil)
    }

    @Test
    func shortPageMeansEndOfLibrary() throws {
        let page = try MediumListMapper.map(
            [payload(id: "11111111-1111-1111-1111-111111111111")],
            pageSize: 50
        )
        #expect(page.nextCursor == nil)
    }

    @Test
    func rejectsInvalidMediumID() {
        #expect(throws: APIMappingError.self) {
            try MediumListMapper.map([payload(id: "not-a-uuid")], pageSize: 1)
        }
    }
}
