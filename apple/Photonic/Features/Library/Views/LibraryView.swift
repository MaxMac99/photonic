import ComposableArchitecture
import SwiftUI

struct LibraryView: View {
    let store: StoreOf<LibraryFeature>

    var body: some View {
        ContentUnavailableView {
            Label("Library", systemImage: "photo.on.rectangle")
        } description: {
            Text("Media browsing lands here.")
        }
        .onAppear {
            store.send(.onAppear)
        }
    }
}

#Preview {
    LibraryView(
        store: Store(initialState: LibraryFeature.State()) {
            LibraryFeature()
        }
    )
}
