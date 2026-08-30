import ComposableArchitecture
import Testing
@testable import Photonic

@MainActor
struct RootFeatureTests {
    @Test
    func tabSelectionUpdatesState() async {
        let store = TestStore(initialState: RootFeature.State()) {
            RootFeature()
        }

        await store.send(.binding(.set(\.currentTab, .settings))) {
            $0.currentTab = .settings
        }
    }
}
