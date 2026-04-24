//
//  DomainError.swift
//  Photonic
//
//  Domain Errors
//

import Foundation

enum DomainError: Error, Equatable {
    case authenticationRequired
    case tokenExpired
    case networkError(String)
    case serverError(code: Int, message: String)
    case validationError(String)
    case notFound(String)
    case unauthorized
    case forbidden
    case conflict(String)
    case decoding(String)
    case encoding(String)
    case unknown(String)
}

extension DomainError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case .authenticationRequired:
            "Authentication is required to perform this action"
        case .tokenExpired:
            "Your session has expired. Please sign in again"
        case let .networkError(message):
            "Network error: \(message)"
        case let .serverError(code, message):
            "Server error (\(code)): \(message)"
        case let .validationError(message):
            "Validation error: \(message)"
        case let .notFound(resource):
            "\(resource) not found"
        case .unauthorized:
            "You are not authorized to perform this action"
        case .forbidden:
            "Access to this resource is forbidden"
        case let .conflict(message):
            "Conflict: \(message)"
        case let .decoding(message):
            "Decoding: \(message)"
        case let .encoding(message):
            "Encoding: \(message)"
        case let .unknown(message):
            "An unknown error occurred: \(message)"
        }
    }
}

enum AuthError: Error, Equatable {
    case noToken
    case invalidToken
    case refreshFailed(String)
    case signInCancelled
    case signInFailed(String)
    case keychainError(String)
    case missingClaim(String)
}

extension AuthError: LocalizedError {
    var errorDescription: String? {
        switch self {
        case .noToken:
            "No authentication token available"
        case .invalidToken:
            "Invalid authentication token"
        case let .refreshFailed(reason):
            "Failed to refresh token: \(reason)"
        case .signInCancelled:
            "Sign in was cancelled"
        case let .signInFailed(reason):
            "Sign in failed: \(reason)"
        case let .keychainError(message):
            "Keychain error: \(message)"
        case let .missingClaim(claim):
            "Missing required claim: \(claim)"
        }
    }
}
