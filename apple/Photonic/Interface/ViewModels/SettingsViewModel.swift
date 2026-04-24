//
//  SettingsViewModel.swift
//  Photonic
//
//  Interface Layer - Settings View Model
//

import Foundation
import SwiftUI

/// View model for the settings screen
///
/// Manages user account information, server configuration, and app settings.
/// Provides real-time data from repositories and handles sign-out functionality.
@MainActor
final class SettingsViewModel: ObservableObject {
    private let logger = LoggerFactory.logger(for: .ui)

    // MARK: - Dependencies

    private let authRepository: AuthRepository
    private let userRepository: UserRepository
    private let serverConfigurationRepository: ServerConfigurationRepository

    // MARK: - Published Properties

    /// Current user account information
    @Published var user: UserAccount?

    /// Current server configuration
    @Published var serverConfiguration: ServerConfiguration?

    /// User statistics (albums, media count, quota)
    @Published var userStats: UserStats?

    /// Loading state for user data
    @Published var isLoadingUser = false

    /// Loading state for server info
    @Published var isLoadingServer = false

    /// Error message for display
    @Published var errorMessage: String?

    /// Connection status
    @Published var isConnected = false

    /// Auto backup setting (persisted in UserDefaults)
    @AppStorage("autoBackupEnabled") var autoBackupEnabled = true

    /// Backup over cellular setting (persisted in UserDefaults)
    @AppStorage("backupOverCellularEnabled") var backupOverCellularEnabled = false

    /// Last backup date (persisted in UserDefaults)
    @AppStorage("lastBackupDate") var lastBackupTimestamp: Double = 0

    // MARK: - Computed Properties

    /// User's email address
    var userEmail: String {
        user?.email ?? "Loading..."
    }

    /// Storage usage text
    var storageUsageText: String {
        guard let stats = userStats else { return "Loading..." }

        let usedGB = formatBytes(Int64(stats.quotaUsed))
        let totalGB = formatBytes(Int64(stats.quota))
        return "\(usedGB) / \(totalGB)"
    }

    /// Storage usage percentage (0-1)
    var storageUsagePercentage: Double {
        guard let stats = userStats, stats.quota > 0 else { return 0 }
        return Double(stats.quotaUsed) / Double(stats.quota)
    }

    /// Total number of photos
    var photoCount: String {
        guard let stats = userStats else { return "0" }
        return NumberFormatter.localizedString(from: NSNumber(value: stats.media), number: .decimal)
    }

    /// Server URL display string
    var serverUrl: String {
        serverConfiguration?.serverUrl.value.host ?? "Not configured"
    }

    /// Server version (mock for now, should come from server info endpoint)
    var serverVersion: String {
        "1.0.0" // TODO: Get from server info endpoint
    }

    /// App version string
    var appVersion: String {
        let version =
            Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "Unknown"
        let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "0"
        return "\(version) (Build \(build))"
    }

    /// Last backup date
    var lastBackupDate: Date? {
        lastBackupTimestamp > 0 ? Date(timeIntervalSince1970: lastBackupTimestamp) : nil
    }

    /// Last backup relative time string
    var lastBackupText: String {
        guard let date = lastBackupDate else { return "Never" }

        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .abbreviated
        return formatter.localizedString(for: date, relativeTo: Date())
    }

    // MARK: - Initialization

    init(
        authRepository: AuthRepository,
        userRepository: UserRepository,
        serverConfigurationRepository: ServerConfigurationRepository
    ) {
        self.authRepository = authRepository
        self.userRepository = userRepository
        self.serverConfigurationRepository = serverConfigurationRepository
    }

    // MARK: - Public Methods

    /// Loads all settings data
    func loadData() async {
        logger.info("Starting to refresh settings data")

        await withTaskGroup(of: Void.self) { group in
            group.addTask { await self.loadUserData() }
            group.addTask { await self.loadServerConfiguration() }
        }

        logger.info("Settings data refresh completed")
    }

    /// Loads user account and statistics
    func loadUserData() async {
        logger.debug("Loading user data")
        isLoadingUser = true
        errorMessage = nil

        do {
            // Load user stats (which includes basic user info)
            let stats = try await userRepository.getUserStats()
            let account = try await authRepository.getUserAccount()

            // Update published properties - this will trigger UI updates
            userStats = stats
            user = account
            isConnected = true
            isLoadingUser = false

            logger.info(
                "User data loaded successfully - Email: \(account.email), Media: \(stats.media)"
            )
        } catch {
            isLoadingUser = false
            isConnected = false
            errorMessage = "Failed to load user data: \(error.localizedDescription)"
            logger.error("Failed to load user data", error: error)
        }
    }

    /// Loads server configuration
    func loadServerConfiguration() async {
        logger.debug("Loading server configuration")
        isLoadingServer = true

        do {
            let config = try await serverConfigurationRepository.getCurrentConfiguration()

            // Update published property - this will trigger UI updates
            serverConfiguration = config
            isLoadingServer = false

            logger.info(
                "Server configuration loaded: \(String(describing: config?.serverUrl.value))"
            )
        } catch {
            isLoadingServer = false
            errorMessage = "Failed to load server configuration: \(error.localizedDescription)"
            logger.error("Failed to load server configuration", error: error)
        }
    }

    /// Signs out the current user
    func signOut() async throws {
        try await authRepository.signOut()

        // Clear stored configuration
        try await serverConfigurationRepository.deleteConfiguration()

        // Clear user data
        user = nil
        userStats = nil
        serverConfiguration = nil

        // Reset backup settings
        lastBackupTimestamp = 0
    }

    /// Updates the last backup timestamp
    func updateLastBackupDate() {
        lastBackupTimestamp = Date().timeIntervalSince1970
    }

    // MARK: - Private Methods

    /// Formats bytes to human-readable string
    private func formatBytes(_ bytes: Int64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .decimal
        formatter.allowedUnits = [.useGB, .useMB]
        return formatter.string(fromByteCount: bytes)
    }
}
