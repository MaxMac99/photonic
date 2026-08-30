import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicCore

/// Persists the user's server configuration. Written by Settings, read
/// app-wide (auth, backup). Moves to PhotonicCore when features become
/// modules (R6).
@DependencyClient
struct ServerConfigurationClient: Sendable {
    var load: @Sendable () async -> ServerConfiguration? = { nil }
    var save: @Sendable (ServerConfiguration) async throws -> Void
    var clear: @Sendable () async -> Void = {}
}

extension DependencyValues {
    var serverConfigurationClient: ServerConfigurationClient {
        get { self[ServerConfigurationClient.self] }
        set { self[ServerConfigurationClient.self] = newValue }
    }
}
