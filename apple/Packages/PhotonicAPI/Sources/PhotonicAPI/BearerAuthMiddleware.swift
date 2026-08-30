import Foundation
import HTTPTypes
import OpenAPIRuntime

/// Bearer-token client middleware. The token provider is evaluated per
/// request so callers can hand it the current session's token.
public struct BearerAuthMiddleware: ClientMiddleware {
    private let accessToken: @Sendable () -> String?

    public init(accessToken: @escaping @Sendable () -> String?) {
        self.accessToken = accessToken
    }

    public func intercept(
        _ request: HTTPRequest,
        body: HTTPBody?,
        baseURL: URL,
        operationID: String,
        next: @Sendable (HTTPRequest, HTTPBody?, URL) async throws -> (HTTPResponse, HTTPBody?)
    ) async throws -> (HTTPResponse, HTTPBody?) {
        var request = request
        if let token = accessToken() {
            request.headerFields[.authorization] = "Bearer \(token)"
        }
        return try await next(request, body, baseURL)
    }
}
