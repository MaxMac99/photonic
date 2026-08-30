import Foundation
import PhotonicCore
import Testing
@testable import Photonic

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
        #expect(
            items.first(where: { $0.name == "code_challenge" })?.value == PKCE.challenge(for: verifier)
        )
        #expect(items.first(where: { $0.name == "code_challenge_method" })?.value == "S256")
        #expect(items.first(where: { $0.name == "state" })?.value == state)
    }
}
