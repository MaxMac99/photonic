//
//  LoggingMiddleware.swift
//  Photonic
//
//  Infrastructure - API Request/Response Logging Middleware
//

import Foundation
import OpenAPIRuntime
import HTTPTypes

/// Middleware that logs all API requests and responses
struct LoggingMiddleware: ClientMiddleware {
    
    private let logger = LoggerFactory.logger(for: .api)
    
    func intercept(
        _ request: HTTPRequest,
        body: HTTPBody?,
        baseURL: URL,
        operationID: String,
        next: @Sendable (HTTPRequest, HTTPBody?, URL) async throws -> (HTTPResponse, HTTPBody?)
    ) async throws -> (HTTPResponse, HTTPBody?) {
        
        let requestId = UUID().uuidString.prefix(8)
        let startTime = Date()
        
        // Log request (without consuming body)
        logRequest(request, baseURL: baseURL, operationID: operationID, requestId: String(requestId))
        
        do {
            // Execute the request
            let (response, responseBody) = try await next(request, body, baseURL)
            
            // Log response (without consuming body)
            let duration = Date().timeIntervalSince(startTime)
            logResponse(response, duration: duration, requestId: String(requestId))
            
            return (response, responseBody)
        } catch {
            // Log error
            let duration = Date().timeIntervalSince(startTime)
            logger.error("[\(requestId)] Request failed after \(String(format: "%.3f", duration))s", error: error)
            throw error
        }
    }
    
    private func logRequest(_ request: HTTPRequest, baseURL: URL, operationID: String, requestId: String) {
        let method = request.method.rawValue
        let path = request.path ?? "/"
        let fullURL = baseURL.absoluteString + path
        
        logger.info("[\(requestId)] 🚀 \(method) \(fullURL)")
        logger.debug("[\(requestId)] Operation: \(operationID)")
        
        // Log headers (excluding sensitive ones)
        #if DEBUG
        var headers: [String: String] = [:]
        for field in request.headerFields {
            let key = field.name.rawName
            let value = field.value
            
            // Mask sensitive headers
            if key.lowercased() == "authorization" {
                headers[key] = "Bearer [REDACTED]"
            } else if key.lowercased().contains("token") || key.lowercased().contains("key") {
                headers[key] = "[REDACTED]"
            } else {
                headers[key] = value
            }
        }
        
        if !headers.isEmpty {
            logger.debug("[\(requestId)] Headers: \(headers)")
        }
        
        // Note: We don't log request body to avoid consuming the HTTPBody stream
        #endif
    }
    
    private func logResponse(_ response: HTTPResponse, duration: TimeInterval, requestId: String) {
        let status = response.status.code
        let statusEmoji = statusEmoji(for: status)
        
        logger.info("[\(requestId)] \(statusEmoji) \(status) - \(String(format: "%.3f", duration))s")
        
        #if DEBUG
        // Log response headers
        var headers: [String: String] = [:]
        for field in response.headerFields {
            let key = field.name.rawName
            let value = field.value
            
            // Skip large or uninteresting headers
            if key.lowercased() != "date" && 
               !key.lowercased().contains("content-encoding") &&
               !key.lowercased().contains("cache") {
                headers[key] = value
            }
        }
        
        if !headers.isEmpty {
            logger.debug("[\(requestId)] Response Headers: \(headers)")
        }
        
        // Note: We don't log response body to avoid consuming the HTTPBody stream
        // The body needs to be available for the actual response handling
        #endif
    }
    
    private func statusEmoji(for code: Int) -> String {
        switch code {
        case 200..<300:
            return "✅"
        case 300..<400:
            return "↪️"
        case 400..<500:
            return "⚠️"
        case 500..<600:
            return "❌"
        default:
            return "❓"
        }
    }
}