import ComposableArchitecture
import SwiftUI

@Reducer
struct RootFeature {
    enum Tab: Hashable {
        case backup
        case library
        case settings
    }

    @ObservableState
    struct State: Equatable {
        var currentTab: Tab = .backup
        var backup = BackupFeature.State()
        var library = LibraryFeature.State()
        var settings = SettingsFeature.State()
    }

    enum Action: BindableAction {
        case binding(BindingAction<State>)
        case backup(BackupFeature.Action)
        case library(LibraryFeature.Action)
        case settings(SettingsFeature.Action)
    }

    var body: some ReducerOf<Self> {
        BindingReducer()
        Scope(state: \.backup, action: \.backup) {
            BackupFeature()
        }
        Scope(state: \.library, action: \.library) {
            LibraryFeature()
        }
        Scope(state: \.settings, action: \.settings) {
            SettingsFeature()
        }
        Reduce { _, action in
            switch action {
            case .binding:
                return .none
            case .backup, .library, .settings:
                return .none
            }
        }
    }
}

struct RootView: View {
    @Bindable var store: StoreOf<RootFeature>

    var body: some View {
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
