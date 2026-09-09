import ComposableArchitecture
import Foundation
import PhotonicCore
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

    @Test
    func restoringDoesNotDowngradeAuthenticatedState() async {
        var state = RootFeature.State()
        state.isAuthenticated = true

        let store = TestStore(initialState: state) {
            RootFeature()
        }

        await store.send(.sessionRestored(false))
    }
}
