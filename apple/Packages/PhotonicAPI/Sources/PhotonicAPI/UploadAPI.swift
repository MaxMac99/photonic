import Foundation
import OpenAPIRuntime
import PhotonicCore

/// Uploads one medium's original data (`create_medium` with a binary body)
/// and its thumbnail variants (`add_medium_item` preview items).
public enum UploadAPI {
    /// Creates the medium from its original file and returns the
    /// server-assigned medium id.
    public static func createMedium(
        serverURL: URL,
        accessToken: String?,
        filename: String,
        dateTaken: Date?,
        data: Data
    ) async throws -> UUID {
        let client = APIClientFactory.makeAuthenticatedClient(
            serverURL: serverURL,
            accessToken: accessToken
        )
        let response = try await client.create_medium(
            Operations.create_medium.Input(
                query: .init(
                    filename: filename,
                    date_taken: dateTaken
                ),
                body: .any(HTTPBody(data))
            )
        )
        switch response {
        case let .created(created):
            guard let mediumID = try? UUID(uuidString: created.body.json) else {
                throw APIMappingError.invalidPayload("medium upload returned no valid id")
            }
            return mediumID
        case .undocumented:
            throw APIMappingError.invalidPayload("upload response was not documented")
        }
    }

    /// One thumbnail variant uploaded as a preview item of an
    /// already-created medium.
    public struct PreviewItemUpload: Sendable {
        public let mediumID: UUID
        public let variant: ThumbnailVariant
        public let filename: String
        public let width: Int
        public let height: Int
        public let data: Data

        public init(
            mediumID: UUID,
            variant: ThumbnailVariant,
            filename: String,
            width: Int,
            height: Int,
            data: Data
        ) {
            self.mediumID = mediumID
            self.variant = variant
            self.filename = filename
            self.width = width
            self.height = height
            self.data = data
        }
    }

    /// Uploads one thumbnail variant as a preview item of an
    /// already-created medium and returns the new item's id.
    public static func addPreviewItem(
        serverURL: URL,
        accessToken: String?,
        upload: PreviewItemUpload
    ) async throws -> UUID {
        let client = APIClientFactory.makeAuthenticatedClient(
            serverURL: serverURL,
            accessToken: accessToken
        )
        let response = try await client.add_medium_item(
            Operations.add_medium_item.Input(
                path: .init(medium_id: upload.mediumID.uuidString, format: .preview),
                query: .init(
                    filename: upload.filename,
                    variant: Operations.add_medium_item.Input.Query.variantPayload(
                        rawValue: upload.variant.rawValue
                    ),
                    width: Int32(upload.width),
                    height: Int32(upload.height)
                ),
                body: .any(HTTPBody(upload.data))
            )
        )
        switch response {
        case let .created(created):
            guard let itemID = try? UUID(uuidString: created.body.json) else {
                throw APIMappingError.invalidPayload("thumbnail upload returned no valid id")
            }
            return itemID
        case .undocumented:
            throw APIMappingError.invalidPayload("thumbnail upload response was not documented")
        }
    }
}
