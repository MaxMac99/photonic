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

    private func makeSession() throws -> AuthSession {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        return try AuthSession(
            serverURL: serverURL,
            serverInfo: ServerInfo(
                version: "1.2.3",
                clientID: "photonic-ios",
                tokenURL: serverURL,
                authorizeURL: serverURL
            ),
            accessToken: #require(
                AccessToken(value: "access", expiresAt: .distantFuture)
            ),
            refreshToken: #require(RefreshToken(value: "refresh"))
        )
    }

    @Test
    func signInWithoutServerConfigurationFails() async {
        let store = TestStore(initialState: AuthFeature.State()) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.load = { nil }
        }

        await store.send(.signInTapped) {
            $0.isSigningIn = true
        }
        await store.receive(.signInFailed("Connect a server in Settings first")) {
            $0.isSigningIn = false
            $0.errorMessage = "Connect a server in Settings first"
        }
    }

    @Test
    func signInSucceedsAndNotifiesParent() async throws {
        let configuration = try ServerConfiguration(
            serverURL: #require(ServerURL("https://photonic.example.com"))
        )
        let session = try makeSession()

        let store = TestStore(initialState: AuthFeature.State()) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.load = { configuration }
            $0.discoveryClient.fetchSystemInfo = { _ in session.serverInfo }
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
        let configuration = try ServerConfiguration(
            serverURL: #require(ServerURL("https://photonic.example.com"))
        )
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = ServerInfo(
            version: "1.2.3",
            clientID: "photonic-ios",
            tokenURL: serverURL,
            authorizeURL: serverURL
        )

        let store = TestStore(initialState: AuthFeature.State()) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient.load = { configuration }
            $0.discoveryClient.fetchSystemInfo = { _ in info }
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

struct PKCETests {
    @Test
    func verifierHasExpectedLengthAndCharset() {
        let verifier = PKCE.makeVerifier()
        #expect(verifier.count == 43)
        let allowed = Set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_")
        #expect(verifier.allSatisfy { allowed.contains($0) })
    }

    @Test
    func challengeMatchesRFC7636Vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
        #expect(PKCE.challenge(for: verifier) == "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM")
    }

    @Test
    func authorizeURLCarriesPKCEParameters() throws {
        let serverURL = try #require(ServerURL("https://photonic.example.com"))
        let info = try ServerInfo(
            version: "1.2.3",
            clientID: "photonic-ios",
            tokenURL: serverURL,
            authorizeURL: #require(ServerURL("https://photonic.example.com/oauth/authorize"))
        )
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
        let state = "state-123"

        let url = try #require(PKCE.authorizeURL(
            info: info,
            verifier: verifier,
            redirectURI: PKCE.redirectURI,
            state: state
        ))
        let items = try #require(URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems)

        #expect(items.first(where: { $0.name == "response_type" })?.value == "code")
        #expect(items.first(where: { $0.name == "client_id" })?.value == "photonic-ios")
        #expect(items.first(where: { $0.name == "redirect_uri" })?.value == PKCE.redirectURI)
        #expect(items.first(where: { $0.name == "code_challenge" })?.value == PKCE.challenge(for: verifier))
        #expect(items.first(where: { $0.name == "code_challenge_method" })?.value == "S256")
        #expect(items.first(where: { $0.name == "state" })?.value == state)
    }
}
