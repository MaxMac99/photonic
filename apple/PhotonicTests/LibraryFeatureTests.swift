import ComposableArchitecture
import Foundation
import PhotonicCore
import Testing
@testable import Photonic

@MainActor
struct LibraryFeatureTests {
    private struct TestFetchFailure: Error, LocalizedError {
        var errorDescription: String? {
            "network unreachable"
        }
    }

    private func medium(_ id: UUID, takenAt: Date? = nil) -> Medium {
        Medium(
            id: id,
            type: .photo,
            albumID: nil,
            takenAt: takenAt,
            primaryFilename: "IMG_0001.HEIC",
            primaryFilesize: 1024
        )
    }

    @Test
    func firstPageAppendsMediaAndContinuesPaging() async {
        let media = [medium(UUID()), medium(UUID())]
        let expectedCursor = MediaCursor(lastDate: nil, lastID: media[1].id)
        var receivedCursors: [MediaCursor?] = []

        let store = TestStore(initialState: LibraryFeature.State()) {
            LibraryFeature()
        } withDependencies: {
            $0.mediaClient.fetchPage = { passedCursor, pageSize in
                receivedCursors.append(passedCursor)
                #expect(pageSize == 50)
                return MediaPage(media: media, nextCursor: expectedCursor)
            }
        }

        await store.send(.onAppear) {
            $0.isLoading = true
        }
        await store.receive(.mediaPageLoaded(MediaPage(media: media, nextCursor: expectedCursor))) {
            $0.isLoading = false
            $0.media = media
            $0.hasMore = true
        }
        #expect(receivedCursors == [nil])
    }

    @Test
    func nextPagePassesCursorAndStopsOnShortPage() async {
        let firstPageMedia = [medium(UUID(), takenAt: Date(timeIntervalSince1970: 100))]
        let secondPageMedia = [medium(UUID(), takenAt: Date(timeIntervalSince1970: 50))]
        let cursorAfterFirst = MediaCursor(
            lastDate: firstPageMedia[0].takenAt,
            lastID: firstPageMedia[0].id
        )

        let store = TestStore(
            initialState: LibraryFeature.State(
                media: firstPageMedia,
                hasMore: true,
                isLoading: false,
                errorMessage: nil
            )
        ) {
            LibraryFeature()
        } withDependencies: {
            $0.mediaClient.fetchPage = { cursor, _ in
                #expect(cursor == cursorAfterFirst)
                return MediaPage(media: secondPageMedia, nextCursor: nil)
            }
        }

        await store.send(.loadNextPage) {
            $0.isLoading = true
        }
        await store.receive(.mediaPageLoaded(MediaPage(media: secondPageMedia, nextCursor: nil))) {
            $0.isLoading = false
            $0.media = firstPageMedia + secondPageMedia
            $0.hasMore = false
        }
    }

    @Test
    func loadingFailureSurfacesErrorAndAllowsRetry() async {
        let store = TestStore(initialState: LibraryFeature.State()) {
            LibraryFeature()
        } withDependencies: {
            $0.mediaClient.fetchPage = { _, _ in throw TestFetchFailure() }
        }

        await store.send(.onAppear) {
            $0.isLoading = true
        }
        await store.receive(.loadingFailed("network unreachable")) {
            $0.isLoading = false
            $0.errorMessage = "network unreachable"
        }
    }

    @Test
    func loadMoreIsIgnoredWhileLoading() async {
        struct UnexpectedFetch: Error {}
        var fetchWasCalled = false

        let store = TestStore(initialState: LibraryFeature.State(isLoading: true)) {
            LibraryFeature()
        } withDependencies: {
            $0.mediaClient.fetchPage = { _, _ in
                fetchWasCalled = true
                throw UnexpectedFetch()
            }
        }

        await store.send(.loadNextPage)
        #expect(!fetchWasCalled)
    }
}
