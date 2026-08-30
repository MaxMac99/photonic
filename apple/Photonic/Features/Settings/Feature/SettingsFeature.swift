import ComposableArchitecture
import Foundation

/// Server selection and app settings.
@Reducer
struct SettingsFeature {
    @ObservableState
    struct State: Equatable {
        var isLoaded = false
    }

    enum Action {
        case onAppear
    }

    var body: some ReducerOf<Self> {
        Reduce { state, action in
            switch action {
            case .onAppear:
                state.isLoaded = true
                return .none
            }
        }
    }
}
