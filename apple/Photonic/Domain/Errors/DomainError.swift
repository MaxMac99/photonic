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
            return "Authentication is required to perform this action"
        case .tokenExpired:
            return "Your session has expired. Please sign in again"
        case .networkError(let message):
            return "Network error: \(message)"
        case .serverError(let code, let message):
            return "Server error (\(code)): \(message)"
        case .validationError(let message):
            return "Validation error: \(message)"
        case .notFound(let resource):
            return "\(resource) not found"
        case .unauthorized:
            return "You are not authorized to perform this action"
        case .forbidden:
            return "Access to this resource is forbidden"
        case .conflict(let message):
            return "Conflict: \(message)"
        case .decoding(let message):
            return "Decoding: \(message)"
        case .encoding(let message):
            return "Encoding: \(message)"
        case .unknown(let message):
            return "An unknown error occurred: \(message)"
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
            return "No authentication token available"
        case .invalidToken:
            return "Invalid authentication token"
        case .refreshFailed(let reason):
            return "Failed to refresh token: \(reason)"
        case .signInCancelled:
            return "Sign in was cancelled"
        case .signInFailed(let reason):
            return "Sign in failed: \(reason)"
        case .keychainError(let message):
            return "Keychain error: \(message)"
        case .missingClaim(let claim):
            return "Missing required claim: \(claim)"
        }
    }
}
