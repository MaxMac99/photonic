import ComposableArchitecture
import SwiftUI

@main
struct PhotonicApp: App {
    static let store = Store(initialState: RootFeature.State()) {
        RootFeature()
    }

    init() {
        CompositionRoot.registerLiveDependencies()
    }

    var body: some Scene {
        WindowGroup {
            RootView(store: PhotonicApp.store)
        }
    }
}
