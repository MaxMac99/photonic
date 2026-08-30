import Dependencies
import Foundation
import PhotonicAPI
import PhotonicCore

extension MediaClient: DependencyKey {
    static var liveValue: MediaClient {
        MediaClient(fetchPage: { cursor, pageSize in
            @Dependency(ServerConfigurationClient.self) var serverConfiguration
            @Dependency(AuthClient.self) var auth

            guard let configuration = await serverConfiguration.load() else {
                throw MediaClientError.serverNotConfigured
            }
            let token = await auth.restoreSession()?.accessToken.value

            let client = APIClientFactory.makeAuthenticatedClient(
                serverURL: configuration.serverURL.rawValue,
                accessToken: token
            )
            let response = try await client.get_all_media(
                Operations.get_all_media.Input(
                    query: .init(
                        per_page: Int64(pageSize),
                        page_last_date: cursor?.lastDate,
                        page_last_id: cursor?.lastID.uuidString
                    )
                )
            )
            switch response {
            case let .ok(ok):
                return try MediumListMapper.map(ok.body.json, pageSize: pageSize)
            case .undocumented:
                throw APIMappingError.invalidPayload("media list response was not JSON")
            }
        })
    }
}
