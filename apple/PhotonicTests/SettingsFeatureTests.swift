import ComposableArchitecture
import Foundation
import PhotonicCore
import Testing
@testable import Photonic

@MainActor
struct SettingsFeatureTests {
    private struct TestDiscoveryFailure: Error, LocalizedError {
        var errorDescription: String? {
            "server unreachable"
        }
    }

    @Test
    func onAppearLoadsSavedConfiguration() async throws {
        let saved = try ServerConfiguration(
            serverURL: #require(ServerURL("https://photonic.example.com"))
        )

        let store = TestStore(initialState: SettingsFeature.State()) {
            SettingsFeature()
        } withDependencies: {
            $0.serverConfigurationClient.load = { saved }
        }

        await store.send(.onAppear)
        await store.receive(.configLoaded(saved)) {
            $0.serverURLText = "https://photonic.example.com"
        }
    }

    @Test
    func connectSavesConfigurationAndDiscoversServer() async throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = ServerInfo(
            version: "1.2.3",
            clientID: "photonic-ios",
            tokenURL: serverURL,
            authorizeURL: serverURL
        )
        var savedConfigurations: [ServerConfiguration] = []

        let store = TestStore(
            initialState: SettingsFeature.State(serverURLText: "https://photonic.example.com")
        ) {
            SettingsFeature()
        } withDependencies: {
            $0.serverConfigurationClient.save = { savedConfigurations.append($0) }
            $0.discoveryClient.fetchSystemInfo = { _ in info }
        }

        await store.send(.connectTapped) {
            $0.isConnecting = true
        }
        await store.receive(.connectionSucceeded(info)) {
            $0.isConnecting = false
            $0.serverInfo = info
            $0.statusMessage = "Connected to Photonic 1.2.3"
        }
        #expect(savedConfigurations == [ServerConfiguration(serverURL: serverURL)])
    }

    @Test
    func connectRejectsInvalidURLWithoutSaving() async {
        var saveCalled = false

        let store = TestStore(
            initialState: SettingsFeature.State(serverURLText: "not a url")
        ) {
            SettingsFeature()
        } withDependencies: {
            $0.serverConfigurationClient.save = { _ in saveCalled = true }
        }

        await store.send(.connectTapped) {
            $0.statusMessage = "Enter a valid server URL (https://…)"
        }
        #expect(!saveCalled)
    }

    @Test
    func discoveryFailureSurfacesError() async {
        let store = TestStore(
            initialState: SettingsFeature.State(serverURLText: "https://photonic.example.com")
        ) {
            SettingsFeature()
        } withDependencies: {
            $0.discoveryClient.fetchSystemInfo = { _ in throw TestDiscoveryFailure() }
        }

        await store.send(.connectTapped) {
            $0.isConnecting = true
        }
        await store.receive(.connectionFailed("server unreachable")) {
            $0.isConnecting = false
            $0.statusMessage = "server unreachable"
        }
    }
}
