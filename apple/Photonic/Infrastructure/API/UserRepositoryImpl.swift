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

        logger.info("User stats fetched successfully (stub implementation)")
        return UserStats(albums: 0, media: 0, quota: 0, quotaUsed: 0)
    }
}
