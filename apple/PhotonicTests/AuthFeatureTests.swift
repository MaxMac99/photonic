import ComposableArchitecture
import Foundation
import PhotonicCore
import Testing
@testable import Photonic

@MainActor
struct AuthFeatureTests {
    private struct TestSignInFailure: Error, LocalizedError {
        var errorDescription: String? {
            "sign-in cancelled"
        }
    }

    private struct TestDiscoveryFailure: Error, LocalizedError {
        var errorDescription: String? {
            "server unreachable"
        }
    }

    private func makeServerInfo(serverURL: ServerURL) -> ServerInfo {
        ServerInfo(
            version: "1.2.3",
            clientID: "photonic-ios",
            tokenURL: serverURL,
            authorizeURL: serverURL
        )
    }

    @Test
    func onAppearConnectsSavedServer() async throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = makeServerInfo(serverURL: serverURL)

        let store = TestStore(initialState: AuthFeature.State()) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.load = {
                ServerConfiguration(serverURL: serverURL)
            }
            $0.discoveryClient.fetchSystemInfo = { _ in info }
        }

        await store.send(.onAppear)
        await store.receive(.configLoaded(ServerConfiguration(serverURL: serverURL))) {
            $0.serverURLText = "https://photonic.example.com"
        }
        await store.receive(.connectionSucceeded(serverURL, info)) {
            $0.connectedServerURL = serverURL
            $0.connectedServer = info
            $0.connectionStatus = "Connected to Photonic 1.2.3"
        }
    }

    @Test
    func connectSavesConfigurationAndDiscoversServer() async throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = makeServerInfo(serverURL: serverURL)
        var savedConfigurations: [ServerConfiguration] = []

        let store = TestStore(
            initialState: AuthFeature.State(serverURLText: "https://photonic.example.com")
        ) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.save = { savedConfigurations.append($0) }
            $0.discoveryClient.fetchSystemInfo = { _ in info }
        }

        await store.send(.connectTapped) {
            $0.isConnecting = true
        }
        await store.receive(.connectionSucceeded(serverURL, info)) {
            $0.isConnecting = false
            $0.connectedServerURL = serverURL
            $0.connectedServer = info
            $0.connectionStatus = "Connected to Photonic 1.2.3"
        }
        #expect(savedConfigurations == [ServerConfiguration(serverURL: serverURL)])
    }

    @Test
    func connectRejectsInvalidURLWithoutSaving() async {
        var saveCalled = false

        let store = TestStore(
            initialState: AuthFeature.State(serverURLText: "not a url")
        ) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.save = { _ in saveCalled = true }
        }

        await store.send(.connectTapped) {
            $0.connectionStatus = "Enter a valid server URL (https://…)"
        }
        #expect(!saveCalled)
    }

    @Test
    func discoveryFailureSurfacesError() async throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))

        let store = TestStore(
            initialState: AuthFeature.State(serverURLText: "https://photonic.example.com")
        ) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.save = { _ in }
            $0.discoveryClient.fetchSystemInfo = { _ in throw TestDiscoveryFailure() }
        }

        await store.send(.connectTapped) {
            $0.isConnecting = true
        }
        await store.receive(.connectionFailed("server unreachable")) {
            $0.isConnecting = false
            $0.connectionStatus = "server unreachable"
        }
    }

    @Test
    func signInWithoutConnectedServerFails() async {
        let store = TestStore(initialState: AuthFeature.State()) {
            AuthFeature()
        }

        await store.send(.signInTapped) {
            $0.errorMessage = "Connect to a server first"
        }
    }

    @Test
    func connectWithOIDCDisabledEntersAppWithoutSignIn() async throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = ServerInfo(
            version: "1.2.3",
            clientID: nil,
            tokenURL: nil,
            authorizeURL: nil
        )
        var signInWasCalled = false

        let store = TestStore(
            initialState: AuthFeature.State(serverURLText: "https://photonic.example.com")
        ) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.save = { _ in }
            $0.discoveryClient.fetchSystemInfo = { _ in info }
            $0.authClient.signIn = { _, _ in
                signInWasCalled = true
                throw TestSignInFailure()
            }
        }

        await store.send(.connectTapped) {
            $0.isConnecting = true
        }
        await store.receive(.connectionSucceeded(serverURL, info)) {
            $0.isConnecting = false
            $0.connectedServerURL = serverURL
            $0.connectedServer = info
            $0.connectionStatus = "Connected to Photonic 1.2.3"
        }
        await store.receive(.delegate(.signInCompleted))
        #expect(!signInWasCalled)
    }

    @Test
    func signInSucceedsAndNotifiesParent() async throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = makeServerInfo(serverURL: serverURL)
        let session = try AuthSession(
            serverURL: serverURL,
            serverInfo: info,
            accessToken: #require(
                AccessToken(value: "access", expiresAt: .distantFuture)
            ),
            refreshToken: #require(RefreshToken(value: "refresh"))
        )

        let store = TestStore(
            initialState: AuthFeature.State(
                connectedServerURL: serverURL,
                connectedServer: info
            )
        ) {
            AuthFeature()
        } withDependencies: {
            $0.authClient.signIn = { _, _ in session }
        }

        await store.send(.signInTapped) {
            $0.isSigningIn = true
        }
        await store.receive(.signInSucceeded(session)) {
            $0.isSigningIn = false
        }
        await store.receive(.delegate(.signInCompleted))
    }

    @Test
    func signInFailureSurfacesError() async throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = makeServerInfo(serverURL: serverURL)

        let store = TestStore(
            initialState: AuthFeature.State(
                connectedServerURL: serverURL,
                connectedServer: info
            )
        ) {
            AuthFeature()
        } withDependencies: {
            $0.authClient.signIn = { _, _ in throw TestSignInFailure() }
        }

        await store.send(.signInTapped) {
            $0.isSigningIn = true
        }
        await store.receive(.signInFailed("sign-in cancelled")) {
            $0.isSigningIn = false
            $0.errorMessage = "sign-in cancelled"
        }
    }
}
