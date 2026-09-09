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
    public static func make(
        serverURL: URL,
        middlewares: [any ClientMiddleware] = []
    ) -> Client {
        Client(
            serverURL: serverURL,
            transport: URLSessionTransport(),
            middlewares: middlewares
        )
    }

    /// Creates a client that carries the session's bearer token, if any.
    public static func makeAuthenticatedClient(
        serverURL: URL,
        accessToken: String?
    ) -> Client {
        var middlewares: [any ClientMiddleware] = []
        if let accessToken {
            middlewares.append(BearerAuthMiddleware(accessToken: { accessToken }))
        }
        return make(serverURL: serverURL, middlewares: middlewares)
    }
}
