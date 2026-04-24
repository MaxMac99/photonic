//
//  CompositionRoot.swift
//  Photonic
//
//  Dependency Injection Composition Root
//

import Foundation
import OpenAPIURLSession
import SwiftData
import SwiftUI

final class CompositionRoot {
    // MARK: - Properties

    private let serverInfo: ServerInfo
    private let apiClient: APIProtocol
    private let authManager: AuthManager

    // MARK: - Repositories (Infrastructure)

    private lazy var serverConfigRepository: ServerConfigurationRepository = ServerConfigurationRepositoryImpl()

    private lazy var authRepository: AuthRepository = AuthRepositoryImpl(authManager: authManager)

    private lazy var mediaRepository: MediaRepository = MediaRepositoryImpl(apiClient: apiClient)

    private lazy var albumRepository: AlbumRepository = AlbumRepositoryImpl(apiClient: apiClient)

    private lazy var userRepository: UserRepository = UserRepositoryImpl(apiClient: apiClient)

    private lazy var photoLibraryAdapter = PhotoLibraryAdapter()

    // MARK: - Services (Infrastructure)

    private lazy var backupService: BackupServiceProtocol = BackupService(
        mediaRepository: mediaRepository,
        albumRepository: albumRepository,
        photoLibraryAdapter: photoLibraryAdapter
    )

    lazy var discoverServerUseCase: DiscoverServerUseCaseProtocol = DiscoverServerUseCase(
        serverConfigurationRepository: serverConfigRepository
    )

    // MARK: - Initialization

    init(serverInfo: ServerInfo) {
        self.serverInfo = serverInfo
        authManager = AuthManager(
            clientId: serverInfo.clientId,
            authorizeUrl: serverInfo.authorizationUrl,
            tokenUrl: serverInfo.tokenUrl
        )

        // Create API client with logging and auth middleware
        apiClient = Client(
            serverURL: serverInfo.serverUrl,
            transport: URLSessionTransport(),
            middlewares: [
                LoggingMiddleware(), // Log requests/responses first
                AuthMiddleware(manager: authManager) // Then add auth
            ]
        )
    }

    // MARK: - Factory Methods for ViewModels

    @MainActor func makeMainViewModel() -> MainViewModel {
        MainViewModel(
            backupService: backupService
        )
    }

    @MainActor func makeMediaViewModel() -> MediaViewModel {
        MediaViewModel(
            mediaRepository: mediaRepository,
            albumRepository: albumRepository
        )
    }

    @MainActor func makeBackupViewModel() -> BackupViewModel {
        BackupViewModel(
            backupService: backupService,
            albumRepository: albumRepository
        )
    }

    @MainActor func makeSettingsViewModel() -> SettingsViewModel {
        let logger = LoggerFactory.logger(for: .application)
        logger.warning("🔴 Creating NEW SettingsViewModel instance")
        return SettingsViewModel(
            authRepository: authRepository,
            userRepository: userRepository,
            serverConfigurationRepository: serverConfigRepository
        )
    }

    @MainActor func makeImageSelectionViewModel() -> ImageSelectionViewModel {
        ImageSelectionViewModel(
            photoLibraryAdapter: photoLibraryAdapter
        )
    }

    @MainActor func makeBackupAlbumSelectionViewModel(modelContext: ModelContext)
        -> BackupAlbumSelectionViewModel
    {
        let backupSelectionRepository = BackupSelectionRepositoryImpl(modelContext: modelContext)
        return BackupAlbumSelectionViewModel(
            backupSelectionRepository: backupSelectionRepository,
            photoLibraryAdapter: photoLibraryAdapter,
            backupService: backupService
        )
    }

    // MARK: - Static Factory Methods for Setup Phase

    /// Creates a DiscoverServerUseCase for initial setup when no server is configured yet
    static func makeSetupDiscoverServerUseCase() -> DiscoverServerUseCaseProtocol {
        let serverConfigRepo = ServerConfigurationRepositoryImpl()
        return DiscoverServerUseCase(
            serverConfigurationRepository: serverConfigRepo
        )
    }
}

// MARK: - Environment Key

struct CompositionRootKey: EnvironmentKey {
    static let defaultValue: CompositionRoot? = nil
}

extension EnvironmentValues {
    var compositionRoot: CompositionRoot? {
        get { self[CompositionRootKey.self] }
        set { self[CompositionRootKey.self] = newValue }
    }
}
