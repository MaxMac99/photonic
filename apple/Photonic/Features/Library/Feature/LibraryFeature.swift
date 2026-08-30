import ComposableArchitecture
import Foundation

/// Browses the backed-up media library. Paged from durable storage per R13.
@Reducer
struct LibraryFeature {
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
