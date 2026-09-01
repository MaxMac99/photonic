//
//  DiscoverServerUseCase.swift
//  Photonic
//
//  Application Use Case
//

import Foundation
import OpenAPIURLSession

protocol DiscoverServerUseCaseProtocol {
    func discoverAndConnect(urlString: String) async throws -> ServerConfiguration
    func validateAndNormalizeUrl(_ urlString: String) -> URL?
}

final class DiscoverServerUseCase: DiscoverServerUseCaseProtocol {
    private let logger = LoggerFactory.logger(for: .application)
    private let serverConfigurationRepository: ServerConfigurationRepository

    init(serverConfigurationRepository: ServerConfigurationRepository) {
        self.serverConfigurationRepository = serverConfigurationRepository
    }

    func discoverAndConnect(urlString: String) async throws -> ServerConfiguration {
        logger.info("Starting server discovery for URL: \(urlString)")

        guard let url = validateAndNormalizeUrl(urlString) else {
            logger.error("Invalid server URL: \(urlString)")
            throw DomainError.validationError("Invalid server URL")
        }

        logger.debug("Normalized URL: \(url)")

        logger.info("Discovering server info...")
        let discoveryInfo = try await serverConfigurationRepository.discoverServerInfo(url: url)

        logger.debug(
            "Discovery info - ClientID: \(discoveryInfo.clientId ?? "none"), TokenURL: \(discoveryInfo.tokenUrl?.absoluteString ?? "none")"
        )

        guard
            let configuration = ServerConfiguration(
                serverUrl: url,
                clientId: discoveryInfo.clientId,
                tokenUrl: discoveryInfo.tokenUrl,
                authorizationUrl: discoveryInfo.authorizeUrl
            )
        else {
            logger.error("Failed to create server configuration from discovery info")
            throw DomainError.validationError("Invalid server configuration")
        }

        logger.info("Saving server configuration...")
        try await serverConfigurationRepository.saveConfiguration(configuration)

        if discoveryInfo.hasOAuth {
            try await connectWithAuthentication(configuration: configuration)
        } else {
            try await connectWithoutAuthentication(serverUrl: url)
        }

        logger.info("Server discovery and connection completed successfully")
        return configuration
    }

    /// Full flow for servers with OIDC enabled: interactive sign-in followed by
    /// validation via user stats.
    private func connectWithAuthentication(configuration: ServerConfiguration) async throws {
        guard let clientId = configuration.clientId,
              let authorizationUrl = configuration.authorizationUrl,
              let tokenUrl = configuration.tokenUrl
        else {
            throw DomainError.validationError("OAuth configuration missing")
        }

        // Create temporary auth manager and repositories to validate the configuration
        logger.info("Creating auth manager for validation...")
        let authManager = AuthManager(
            clientId: clientId,
            authorizeUrl: authorizationUrl,
            tokenUrl: tokenUrl
        )

        let authRepository = AuthRepositoryImpl(authManager: authManager)

        // Create API client for validation
        logger.debug("Creating API client with server URL: \(configuration.serverUrl.value)")
        let apiClient = Client(
            serverURL: configuration.serverUrl.value,
            transport: URLSessionTransport(),
            middlewares: [
                LoggingMiddleware(),
                AuthMiddleware(manager: authManager)
            ]
        )

        let userRepository = UserRepositoryImpl(apiClient: apiClient)

        // Perform authentication and validate by fetching user stats
        logger.info("Performing interactive sign-in...")
        _ = try await authRepository.signInInteractive()
        logger.info("Sign-in successful, received tokens")

        logger.info("Validating connection by fetching user stats...")
        let stats = try await userRepository.getUserStats()
        logger.info("Validation successful - User has \(stats.media) media items")
    }

    /// Flow for servers with OIDC disabled: connect without authentication and
    /// validate the connection by fetching the public server info.
    private func connectWithoutAuthentication(serverUrl: URL) async throws {
        logger.info("Server has no OAuth configured - connecting without authentication")
        let apiClient = Client(
            serverURL: serverUrl,
            transport: URLSessionTransport(),
            middlewares: [LoggingMiddleware()]
        )

        logger.info("Validating connection by fetching server info...")
        _ = try await apiClient.system_info()
    }

    func validateAndNormalizeUrl(_ urlString: String) -> URL? {
        var normalizedString = urlString.trimmingCharacters(in: .whitespacesAndNewlines)
        if !normalizedString.contains("://") {
            normalizedString = "https://\(normalizedString)"
        }

        guard var components = URLComponents(string: normalizedString) else {
            return nil
        }

        if components.scheme == nil {
            components.scheme = "https"
        }

        guard let scheme = components.scheme?.lowercased(),
              ["http", "https"].contains(scheme)
        else {
            return nil
        }

        if components.port == nil, scheme == "http" {
            components.port = 8080
        }

        return components.url
    }
}
