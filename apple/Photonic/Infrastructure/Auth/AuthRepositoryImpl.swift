//
//  AuthRepositoryImpl.swift
//  Photonic
//
//  Infrastructure Adapter - Auth Repository Implementation
//

import AuthenticationServices
import Foundation

/// A description
final class AuthRepositoryImpl: AuthRepository {
    private let logger = LoggerFactory.logger(for: .auth)

    private let authManager: AuthManager
    private let keychainService = "com.photonic.oauth"

    init(authManager: AuthManager) {
        self.authManager = authManager
    }

    func getUserAccount() async throws -> UserAccount {
        logger.debug("Fetching user account from token")

        guard let token = await authManager.getCurrentToken() else {
            logger.error("No token available when fetching user account")
            throw AuthError.noToken
        }

        let claims = token.jwt.claims

        guard let email = claims.email else {
            logger.error("Missing email claim in JWT")
            throw AuthError.missingClaim("email")
        }

        var quota: UserAccount.Quota?
        if let quotaString = claims.quota {
            quota = parseQuota(from: quotaString)
        }

        logger.info("Successfully retrieved user account for: \(claims.sub)")

        return UserAccount(
            id: claims.sub,
            email: email,
            name: claims.name,
            givenName: claims.given_name,
            familyName: claims.family_name,
            nickname: claims.nickname,
            preferredUsername: claims.preferred_username,
            profileUrl: claims.profile,
            pictureUrl: claims.picture,
            emailVerified: claims.email_verified ?? false,
            quota: quota,
            createdAt: claims.iat
        )
    }

    private func parseQuota(from quotaString: String) -> UserAccount.Quota? {
        let components = quotaString.split(separator: "/")
        guard components.count == 2,
              let usedBytes = Int64(components[0]),
              let totalBytes = Int64(components[1])
        else {
            return nil
        }

        return UserAccount.Quota(
            totalBytes: totalBytes,
            usedBytes: usedBytes
        )
    }

    func signInInteractive() async throws -> (access: AccessToken, refresh: RefreshToken) {
        logger.info("Starting interactive sign-in")

        let token = try await authManager.getAccessToken()
        logger.debug("Access token obtained")

        let accessToken = AccessToken(
            value: token.raw,
            expiresAt: token.jwt.claims.exp,
            scopes: ["openid", "quota", "offline_access", "email", "profile"]
        )

        guard let refreshTokenString = await authManager.getRefreshToken() else {
            logger.error("No refresh token available after sign-in")
            throw AuthError.noToken
        }

        let refreshToken = RefreshToken(value: refreshTokenString)

        // Store tokens in Keychain
        logger.debug("Storing tokens in keychain")

        if let accessTokenData = accessToken.value.data(using: .utf8) {
            try KeychainHelper.upsertData(
                data: accessTokenData,
                forService: keychainService,
                account: "access_token"
            )
        }

        if let refreshTokenData = refreshToken.value.data(using: .utf8) {
            try KeychainHelper.upsertData(
                data: refreshTokenData,
                forService: keychainService,
                account: "refresh_token"
            )
        }

        let expiresAtString = String(accessToken.expiresAt.timeIntervalSince1970)
        if let expiresAtData = expiresAtString.data(using: .utf8) {
            try KeychainHelper.upsertData(
                data: expiresAtData,
                forService: keychainService,
                account: "token_expires_at"
            )
        }

        logger.info("Sign-in completed successfully")
        return (access: accessToken, refresh: refreshToken)
    }

    func signOut() async throws {
        logger.info("Signing out user")
        try await authManager.signOut()
        logger.info("Sign-out completed")
    }
}
