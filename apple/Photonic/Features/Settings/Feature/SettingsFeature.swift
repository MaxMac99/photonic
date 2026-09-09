import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicAPI
import PhotonicCore

/// Server selection and connection. Validates the URL at the boundary via
/// the `ServerURL` value object and probes the server for its system info.
@Reducer
struct SettingsFeature {
    @Dependency(ServerConfigurationClient.self) private var serverConfiguration
    @Dependency(DiscoveryClient.self) private var discovery

    private enum CancelID {
        case connect
    }

    @ObservableState
    struct State: Equatable {
        var serverURLText = ""
        var statusMessage: String?
        var isConnecting = false
        var serverInfo: ServerInfo?
    }

    enum Action: BindableAction, Equatable {
        case binding(BindingAction<State>)
        case onAppear
        case configLoaded(ServerConfiguration)
        case connectTapped
        case connectionSucceeded(ServerInfo)
        case connectionFailed(String)
    }

    var body: some ReducerOf<Self> {
        BindingReducer()
        Reduce { state, action in
            switch action {
            case .onAppear:
                return .run { send in
                    if let configuration = await serverConfiguration.load() {
                        await send(.configLoaded(configuration))
                    }
                }

            case let .configLoaded(configuration):
                state.serverURLText = configuration.serverURL.absoluteString
                return .none

            case .connectTapped:
                guard let serverURL = ServerURL(state.serverURLText) else {
                    state.statusMessage = "Enter a valid server URL (https://…)"
                    return .none
                }
                state.isConnecting = true
                state.statusMessage = nil
                return .run { send in
                    do {
                        try await serverConfiguration.save(ServerConfiguration(serverURL: serverURL))
                        let info = try await discovery.fetchSystemInfo(serverURL)
                        await send(.connectionSucceeded(info))
                    } catch {
                        await send(.connectionFailed(error.localizedDescription))
                    }
                }
                .cancellable(id: CancelID.connect, cancelInFlight: true)

            case let .connectionSucceeded(info):
                state.isConnecting = false
                state.serverInfo = info
                state.statusMessage = "Connected to Photonic \(info.version)"
                return .none

            case let .connectionFailed(message):
                state.isConnecting = false
                state.statusMessage = message
                return .none

            case .binding:
                return .none
            }
        }
    }
}
