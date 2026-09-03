import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicAPI
import PhotonicCore

/// First-run and sign-in flow: connect to a server (if not yet configured),
/// then run the OAuth2+PKCE sign-in. The root feature gates the tab shell
/// behind this feature until a session exists.
@Reducer
struct AuthFeature {
    @Dependency(AuthClient.self) private var auth
    @Dependency(ServerConfigurationClient.self) private var serverConfiguration
    @Dependency(DiscoveryClient.self) private var discovery

    private enum CancelID {
        case connect
    }

    @ObservableState
    struct State: Equatable {
        var serverURLText = ""
        var connectionStatus: String?
        var isConnecting = false
        var connectedServerURL: ServerURL?
        var connectedServer: ServerInfo?
        var isSigningIn = false
        var errorMessage: String?
    }

    enum Action: BindableAction, Equatable {
        case binding(BindingAction<State>)
        case onAppear
        case configLoaded(ServerConfiguration)
        case connectTapped
        case connectionSucceeded(ServerURL, ServerInfo)
        case connectionFailed(String)
        case signInTapped
        case signInSucceeded(AuthSession)
        case signInFailed(String)
        case delegate(Delegate)

        enum Delegate: Equatable {
            case signInCompleted
        }
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
                return connect(configuration.serverURL)

            case .connectTapped:
                guard let serverURL = ServerURL(state.serverURLText) else {
                    state.connectionStatus = "Enter a valid server URL (https://…)"
                    return .none
                }
                state.isConnecting = true
                state.connectionStatus = nil
                return connect(serverURL)

            case let .connectionSucceeded(serverURL, info):
                state.isConnecting = false
                state.connectedServerURL = serverURL
                state.connectedServer = info
                state.connectionStatus = "Connected to Photonic \(info.version)"
                // Servers with OIDC disabled connect without authentication
                // (main's semantics): go straight into the app.
                if info.isOIDCEnabled {
                    return .none
                }
                return .send(.delegate(.signInCompleted))

            case let .connectionFailed(message):
                state.isConnecting = false
                state.connectionStatus = message
                return .none

            case .signInTapped:
                guard let serverURL = state.connectedServerURL,
                      let info = state.connectedServer
                else {
                    state.errorMessage = "Connect to a server first"
                    return .none
                }
                // Nothing interactive to do without OIDC.
                guard info.isOIDCEnabled else {
                    return .send(.delegate(.signInCompleted))
                }
                state.isSigningIn = true
                state.errorMessage = nil
                return .run { send in
                    do {
                        let session = try await auth.signIn(serverURL, info)
                        await send(.signInSucceeded(session))
                    } catch {
                        await send(.signInFailed(error.localizedDescription))
                    }
                }

            case let .signInSucceeded(session):
                state.isSigningIn = false
                return .send(.delegate(.signInCompleted))

            case let .signInFailed(message):
                state.isSigningIn = false
                state.errorMessage = message
                return .none

            case .delegate:
                return .none

            case .binding:
                return .none
            }
        }
    }

    private func connect(_ serverURL: ServerURL) -> Effect<AuthFeature.Action> {
        Effect.run { send in
            do {
                try await serverConfiguration.save(ServerConfiguration(serverURL: serverURL))
                let info = try await discovery.fetchSystemInfo(serverURL)
                await send(.connectionSucceeded(serverURL, info))
            } catch {
                await send(.connectionFailed(error.localizedDescription))
            }
        }
        .cancellable(id: CancelID.connect, cancelInFlight: true)
    }
}
