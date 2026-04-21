//
//  UserRepositoryImpl.swift
//  Photonic
//
//  Infrastructure Layer - API Implementation
//

import Foundation
import OpenAPIURLSession

final class UserRepositoryImpl: UserRepository {

    private let logger = LoggerFactory.logger(for: .api)
    private let apiClient: APIProtocol

    init(apiClient: APIProtocol) {
        self.apiClient = apiClient
    }

    func getUserStats() async throws -> UserStats {
        logger.debug("Fetching user stats")

        do {
            let response = try await apiClient.user_stats(.init())
            let apiStats = try response.ok.body.json

            logger.info(
                "User stats fetched successfully - Albums: \(apiStats.albums), Media: \(apiStats.media)"
            )

            return UserStats(
                albums: UInt64(apiStats.albums),
                media: UInt64(apiStats.media),
                quota: UInt64(apiStats.quota),
                quotaUsed: UInt64(apiStats.quota_used)
            )
        } catch {
            logger.error("Failed to fetch user stats", error: error)
            throw DomainError.networkError(
                "Failed to fetch user stats: \(error.localizedDescription)")
        }
    }
}
