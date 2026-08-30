import Dependencies
import DependenciesMacros
import Foundation
import PhotonicCore

/// Typed interface for server discovery: probing a Photonic server for its
/// version and OAuth endpoints (`system_info`).
@DependencyClient
public struct DiscoveryClient: Sendable {
    public var fetchSystemInfo: @Sendable (ServerURL) async throws -> ServerInfo
}

extension DiscoveryClient: DependencyKey {
    public static var liveValue: DiscoveryClient {
        DiscoveryClient(fetchSystemInfo: { serverURL in
            let client = APIClientFactory.make(serverURL: serverURL.rawValue)
            let response = try await client.system_info(Operations.system_info.Input())
            switch response {
            case let .ok(ok):
                return try ServerInfoMapper.map(ok.body.json)
            case .undocumented:
                throw APIMappingError.invalidPayload("system info response was not JSON")
            }
        })
    }
}

public extension DependencyValues {
    var discoveryClient: DiscoveryClient {
        get { self[DiscoveryClient.self] }
        set { self[DiscoveryClient.self] = newValue }
    }
}
