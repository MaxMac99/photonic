import Foundation
import PhotonicCore
import Testing
@testable import PhotonicAPI

struct ServerInfoMapperTests {
    @Test
    func mapsSystemInfo() throws {
        let payload = Components.Schemas.InfoResponse(
            authorize_url: "https://server.example.com/oauth/authorize",
            client_id: "photonic-ios",
            token_url: "https://server.example.com/oauth/token",
            version: "1.2.3"
        )

        let info = try ServerInfoMapper.map(payload)

        #expect(info.version == "1.2.3")
        #expect(info.clientID == "photonic-ios")
        #expect(info.authorizeURL?.absoluteString == "https://server.example.com/oauth/authorize")
        #expect(info.tokenURL?.absoluteString == "https://server.example.com/oauth/token")
        #expect(info.isOIDCEnabled)
    }

    @Test
    func mapsServerWithoutOIDC() throws {
        let payload = Components.Schemas.InfoResponse(
            authorize_url: nil,
            client_id: nil,
            token_url: nil,
            version: "1.2.3"
        )

        let info = try ServerInfoMapper.map(payload)

        #expect(info.version == "1.2.3")
        #expect(info.clientID == nil)
        #expect(!info.isOIDCEnabled)
    }

    @Test
    func rejectsPartialOAuthConfiguration() {
        #expect(throws: APIMappingError.self) {
            try ServerInfoMapper.map(
                Components.Schemas.InfoResponse(
                    authorize_url: nil,
                    client_id: "photonic-ios",
                    token_url: nil,
                    version: "1.2.3"
                )
            )
        }
    }

    @Test
    func rejectsInvalidOAuthURLs() {
        let payload = Components.Schemas.InfoResponse(
            authorize_url: "not a url",
            client_id: "photonic-ios",
            token_url: "https://server.example.com/oauth/token",
            version: "1.2.3"
        )

        #expect(throws: APIMappingError.self) {
            try ServerInfoMapper.map(payload)
        }
    }
}
