import Foundation
import OpenAPIRuntime
import PhotonicCore

/// Uploads one medium's original data (`create_medium` with a binary body).
public enum UploadAPI {
    public static func createMedium(
        serverURL: URL,
        accessToken: String?,
        filename: String,
        dateTaken: Date?,
        data: Data
    ) async throws {
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
        case .created:
            return
        case .undocumented:
            throw APIMappingError.invalidPayload("upload response was not documented")
        }
    }
}
