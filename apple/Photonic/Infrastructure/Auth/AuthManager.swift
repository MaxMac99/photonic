//
//  AuthManager.swift
//  Photonic
//
//  Created by Max Vissing on 03.05.24.
//

import AuthenticationServices
import Foundation
import OAuth2
import SwiftJWT

struct PhotonicClaims: Claims {
    var iss: String
    var sub: String
    var exp: Date
    var iat: Date
    var auth_time: Date? = nil
    var name: String? = nil
    var given_name: String? = nil
    var family_name: String? = nil
    var nickname: String? = nil
    var preferred_username: String? = nil
    var profile: String? = nil
    var picture: String? = nil
    var email: String? = nil
    var email_verified: Bool? = nil
    var quota: String? = nil
}

struct Token {
    var raw: String
    var jwt: JWT<PhotonicClaims>
}

actor AuthManager {
    
    private let logger = LoggerFactory.logger(for: .auth)
    private var oauth: OAuth2

    init(clientId: String, authorizeUrl: URL, tokenUrl: URL) {
        oauth = OAuth2CodeGrant(settings: [
            "client_id": clientId,
            "authorize_uri": authorizeUrl.absoluteString,
            "token_uri": tokenUrl.absoluteString,
            "redirect_uris": ["photonic://oauth/callback"],
            "scope": "openid quota offline_access email profile",
            "use_pkce": true,
        ])
        oauth.logger = OAuth2DebugLogger(.trace)
        oauth.authConfig.ui.prefersEphemeralWebBrowserSession = true
    }

    private var authorizeTask: Task<Token, Error>?

    func getAccessToken() async throws -> Token {
        logger.debug("Getting access token")
        
        if let authorizeTask {
            logger.debug("Returning existing authorization task")
            return try await authorizeTask.value
        }

        logger.info("Starting new authorization flow")
        
        authorizeTask = Task { () throws -> Token in
            self.oauth.authConfig.authorizeEmbedded = true
            let anchor = await ASPresentationAnchor()
            self.oauth.authConfig.authorizeContext = anchor
            
            self.logger.debug("Initiating OAuth2 authorization")
            
            let raw: String = try await withCheckedThrowingContinuation { continuation in
                self.oauth.authorize { success, error in
                    if let error {
                        self.logger.error("OAuth2 authorization failed", error: error)
                        continuation.resume(throwing: error)
                        return
                    }
                    if let token = self.oauth.accessToken {
                        self.logger.info("OAuth2 authorization successful")
                        continuation.resume(returning: token)
                        return
                    }
                    self.logger.error("OAuth2 completed but no token received")
                    continuation.resume(throwing: AuthError.noToken)
                }
            }
            
            self.logger.debug("Parsing JWT token")
            return Token(raw: raw, jwt: try JWT<PhotonicClaims>(jwtString: raw))
        }
        return try await authorizeTask!.value
    }

    func refreshAccessToken() async throws {
        logger.info("Refreshing access token")
        
        try await withCheckedThrowingContinuation {
            (continuation: CheckedContinuation<Void, Error>) in
            self.oauth.doRefreshToken { success, error in
                if let error {
                    self.logger.error("Token refresh failed", error: error)
                    continuation.resume(throwing: error)
                    return
                }
                self.logger.info("Token refresh successful")
                continuation.resume()
            }
        }
    }

    func getCurrentToken() async -> Token? {
        guard let raw = oauth.accessToken else {
            logger.debug("No current access token available")
            return nil
        }
        guard let jwt = try? JWT<PhotonicClaims>(jwtString: raw) else {
            logger.warning("Failed to parse current JWT token")
            return nil
        }
        logger.debug("Retrieved current token for subject: \(jwt.claims.sub)")
        return Token(raw: raw, jwt: jwt)
    }

    func getRefreshToken() -> String? {
        return oauth.refreshToken
    }

    func signOut() async throws {
        logger.info("Signing out - clearing tokens")
        oauth.forgetTokens()
        authorizeTask = nil
        logger.debug("Tokens cleared successfully")
    }
}
