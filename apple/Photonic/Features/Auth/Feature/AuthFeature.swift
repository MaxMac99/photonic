import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicAPI
import PhotonicCore

/// Interactive sign-in. The root feature gates the tab view behind this
/// feature until a session exists.
@Reducer
struct AuthFeature {
    @Dependency(AuthClient.self) private var auth
    @Dependency(ServerConfigurationClient.self) private var serverConfiguration
    @Dependency(DiscoveryClient.self) private var discovery

    @ObservableState
    struct State: Equatable {
        var isSigningIn = false
        var errorMessage: String?
    }

    enum Action: Equatable {
        case signInTapped
        case signInSucceeded(AuthSession)
        case signInFailed(String)
        case delegate(Delegate)

        enum Delegate: Equatable {
            case signInCompleted
        }
    }

    var body: some ReducerOf<Self> {
        Reduce { state, action in
            switch action {
            case .signInTapped:
                state.isSigningIn = true
                state.errorMessage = nil
                return .run { send in
                    guard let configuration = await serverConfiguration.load() else {
                        await send(.signInFailed("Connect a server in Settings first"))
                        return
                    }
                    do {
                        let info = try await discovery.fetchSystemInfo(configuration.serverURL)
                        let session = try await auth.signIn(configuration.serverURL, info)
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
            }
        }
    }
}
