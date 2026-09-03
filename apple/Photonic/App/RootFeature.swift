import ComposableArchitecture
import Dependencies
import SwiftUI

@Reducer
struct RootFeature {
    @Dependency(AuthClient.self) private var auth

    enum Tab: Hashable {
        case backup
        case library
        case settings
    }

    @ObservableState
    struct State: Equatable {
        var currentTab: Tab = .backup
        var isAuthenticated = false
        var auth = AuthFeature.State()
        var backup = BackupFeature.State()
        var library = LibraryFeature.State()
        var settings = SettingsFeature.State()
    }

    enum Action: BindableAction {
        case binding(BindingAction<State>)
        case onAppear
        case sessionRestored(Bool)
        case auth(AuthFeature.Action)
        case backup(BackupFeature.Action)
        case library(LibraryFeature.Action)
        case settings(SettingsFeature.Action)
    }

    var body: some ReducerOf<Self> {
        BindingReducer()
        Scope(state: \.auth, action: \.auth) {
            AuthFeature()
        }
        Scope(state: \.backup, action: \.backup) {
            BackupFeature()
        }
        Scope(state: \.library, action: \.library) {
            LibraryFeature()
        }
        Scope(state: \.settings, action: \.settings) {
            SettingsFeature()
        }
        Reduce { state, action in
            switch action {
            case .onAppear:
                return .run { send in
                    let session = await auth.restoreSession()
                    await send(.sessionRestored(session != nil))
                }

            case let .sessionRestored(authenticated):
                // The connect flow may already have entered the app (OIDC
                // disabled); restoring must never downgrade that.
                if !state.isAuthenticated {
                    state.isAuthenticated = authenticated
                }
                return .none

            case .auth(.delegate(.signInCompleted)):
                state.isAuthenticated = true
                return .none

            case .auth:
                return .none

            case .backup, .library, .settings:
                return .none

            case .binding:
                return .none
            }
        }
    }
}

struct RootView: View {
    @Bindable var store: StoreOf<RootFeature>

    var body: some View {
        if store.isAuthenticated {
            tabs
        } else {
            AuthView(store: store.scope(state: \.auth, action: \.auth))
                .onAppear {
                    guard !AppRuntime.isRunningUnitTests else { return }
                    store.send(.onAppear)
                }
        }
    }

    private var tabs: some View {
        TabView(selection: $store.currentTab) {
            BackupView(store: store.scope(state: \.backup, action: \.backup))
                .tabItem {
                    Label("Backup", systemImage: "arrow.up.circle")
                }
                .tag(RootFeature.Tab.backup)

            LibraryView(store: store.scope(state: \.library, action: \.library))
                .tabItem {
                    Label("Library", systemImage: "photo.on.rectangle")
                }
                .tag(RootFeature.Tab.library)

            SettingsView(store: store.scope(state: \.settings, action: \.settings))
                .tabItem {
                    Label("Settings", systemImage: "gearshape")
                }
                .tag(RootFeature.Tab.settings)
        }
    }
}
