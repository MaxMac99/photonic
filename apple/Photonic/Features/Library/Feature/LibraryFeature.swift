import ComposableArchitecture
import Dependencies
import Foundation
import PhotonicCore

/// Browses the backed-up media library. Paged from the server via cursor
/// queries; pages stay small and scoped (R13).
@Reducer
struct LibraryFeature {
    @Dependency(MediaClient.self) private var media

    private static let pageSize = 50

    @ObservableState
    struct State: Equatable {
        var media: [Medium] = []
        var hasMore = true
        var isLoading = false
        var errorMessage: String?
    }

    enum Action: Equatable {
        case onAppear
        case loadNextPage
        case mediaPageLoaded(MediaPage)
        case loadingFailed(String)
    }

    var body: some ReducerOf<Self> {
        Reduce { state, action in
            switch action {
            case .onAppear, .loadNextPage:
                guard !state.isLoading, state.hasMore else { return .none }
                state.isLoading = true
                state.errorMessage = nil
                let cursor = state.media.last.map {
                    MediaCursor(lastDate: $0.takenAt, lastID: $0.id)
                }
                return .run { send in
                    do {
                        let page = try await media.fetchPage(cursor, Self.pageSize)
                        await send(.mediaPageLoaded(page))
                    } catch {
                        await send(.loadingFailed(error.localizedDescription))
                    }
                }

            case let .mediaPageLoaded(page):
                state.isLoading = false
                state.media += page.media
                state.hasMore = page.nextCursor != nil
                return .none

            case let .loadingFailed(message):
                state.isLoading = false
                state.errorMessage = message
                return .none
            }
        }
    }
}
