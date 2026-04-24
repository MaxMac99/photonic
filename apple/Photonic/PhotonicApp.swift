//
//  PhotonicApp.swift
//  Photonic
//
//  Created by Max Vissing on 09.02.24.
//

import OpenAPIURLSession
import Photos
import SwiftData
import SwiftUI

#if DEBUG
import XcodebuildNvimPreview
#endif

@main
struct PhotonicApp: App {
    @AppStorage("serverInfo") private var serverInfoData: Data?
    @StateObject private var compositionRoot = CompositionRootContainer()

    private var client: APIProtocol? {
        guard let serverInfoData else {
            return nil
        }
        guard let serverInfo = try? JSONDecoder().decode(
            ServerInfo.self, from: serverInfoData
        ) else {
            return nil
        }
        let authManager = AuthManager(
            clientId: serverInfo.clientId,
            authorizeUrl: serverInfo.authorizationUrl,
            tokenUrl: serverInfo.tokenUrl
        )
        return Client(
            serverURL: serverInfo.serverUrl, transport: URLSessionTransport(),
            middlewares: [
                LoggingMiddleware(),
                AuthMiddleware(manager: authManager)
            ]
        )
    }

    var body: some Scene {
        WindowGroup {
            Group {
                if compositionRoot.serverConfiguration != nil {
                    PhotonicMainView().environment(\.apiClient, client ?? createMockClient()).environment(
                        \.compositionRoot,
                        compositionRoot.root
                    )
                } else {
                    SetupUrlView(
                        viewModel: createSetupViewModel(),
                        onSetupComplete: {
                            config in
                            compositionRoot.setConfiguration(config)
                        }
                    )
                }
            }.modelContainer(for: BackupAlbumSelection.self)
            #if DEBUG
                .setupNvimPreview {
                    NavigationStack {
                        SetupUrlView(viewModel: MockSetupViewModel())
                    }
                }
            #endif
        }
    }

    private func createSetupViewModel() -> SetupViewModel {
        let discoverServerUseCase = CompositionRoot.makeSetupDiscoverServerUseCase()
        return SetupViewModel(discoverServerUseCase: discoverServerUseCase)
    }

    private func createMockClient() -> APIProtocol {
        Client(
            serverURL: URL(string: "http://localhost")!,
            transport: URLSessionTransport()
        )
    }
}

// MARK: - Composition Root Container

@MainActor
final class CompositionRootContainer: ObservableObject {
    @Published var serverConfiguration: ServerConfiguration?

    var root: CompositionRoot? {
        guard let serverInfo = serverConfiguration else {
            return nil
        }
        // Convert ServerConfiguration to ServerInfo for legacy compatibility
        let legacyServerInfo = ServerInfo(
            serverUrl: serverInfo.serverUrl.value,
            clientId: serverInfo.clientId,
            tokenUrl: serverInfo.tokenUrl,
            authorizationUrl: serverInfo.authorizationUrl
        )
        return CompositionRoot(serverInfo: legacyServerInfo)
    }

    init() {
        loadConfiguration()
    }

    func loadConfiguration() {
        // Try to load existing configuration
        if let data = UserDefaults.standard.data(forKey: "de.photonic.serverConfiguration"),
           let config = try? JSONDecoder().decode(ServerConfiguration.self, from: data)
        {
            serverConfiguration = config
        }
    }

    func setConfiguration(_ config: ServerConfiguration) {
        serverConfiguration = config
    }
}

struct ApiClientKey: EnvironmentKey {
    static var defaultValue: APIProtocol = Client(
        serverURL: URL(string: "http://localhost")!,
        transport: URLSessionTransport()
    )
}

extension EnvironmentValues {
    var apiClient: APIProtocol {
        get {
            self[ApiClientKey.self]
        }
        set {
            self[ApiClientKey.self] = newValue
        }
    }
}
