//
//  AuthMiddleware.swift
//  Photonic
//
//  Created by Max Vissing on 02.05.24.
//

import Foundation
import OpenAPIRuntime
import HTTPTypes
@preconcurrency import OAuth2

struct AuthMiddleware: ClientMiddleware {
    
    let manager: AuthManager
    
    func intercept(_ request: HTTPRequest, body: HTTPBody?, baseURL: URL, operationID: String, next: @Sendable (HTTPRequest, HTTPBody?, URL) async throws -> (HTTPResponse, HTTPBody?)) async throws -> (HTTPResponse, HTTPBody?) {
        return try await loadAuthorized(request, body: body, baseURL: baseURL, operationID: operationID, next: next, retry: true)
    }
    
    private func loadAuthorized(_ request: HTTPRequest, body: HTTPBody?, baseURL: URL, operationID: String, next: @Sendable (HTTPRequest, HTTPBody?, URL) async throws -> (HTTPResponse, HTTPBody?), retry: Bool = true) async throws -> (HTTPResponse, HTTPBody?) {
        var request = request
        
        let token = try await manager.getAccessToken()
        request.headerFields[.authorization] = "Bearer \(token.raw)"
        
        let (response, responseBody) = try await next(request, body, baseURL)
        if response.status.code == 401 && retry {
            try await manager.refreshAccessToken()
            return try await loadAuthorized(request, body: body, baseURL: baseURL, operationID: operationID, next: next, retry: false)
        }
        return (response, responseBody)
    }
}
