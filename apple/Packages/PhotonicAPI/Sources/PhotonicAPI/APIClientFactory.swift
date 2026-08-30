import Foundation
import OpenAPIRuntime
import OpenAPIURLSession

/// Factory for the generated Photonic API client.
///
/// This is the only place that touches the OpenAPI transport stack (R3/R5):
/// consumers import `PhotonicAPI` and use the returned `Client` without
/// ever importing OpenAPI modules.
public enum APIClientFactory {
    /// Creates a client for the Photonic server at `serverURL`.
    /// Auth middleware wraps the transport when auth lands (build-order step 5).
    public static func make(serverURL: URL) -> Client {
        Client(
            serverURL: serverURL,
            transport: URLSessionTransport()
        )
    }
}
