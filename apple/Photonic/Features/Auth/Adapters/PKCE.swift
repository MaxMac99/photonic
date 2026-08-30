import CryptoKit
import Foundation
import PhotonicCore
import Security

/// PKCE (RFC 7636) helpers: verifier generation, S256 challenge, and the
/// authorize URL. Pure functions — fully unit tested.
enum PKCE {
    static func makeVerifier() -> String {
        base64URL(randomBytes(count: 32))
    }

    static func makeState() -> String {
        base64URL(randomBytes(count: 16))
    }

    static func challenge(for verifier: String) -> String {
        base64URL(Data(SHA256.hash(data: Data(verifier.utf8))))
    }

    static func authorizeURL(
        info: ServerInfo,
        verifier: String,
        redirectURI: String,
        state: String
    ) -> URL? {
        guard var components = URLComponents(
            url: info.authorizeURL.rawValue,
            resolvingAgainstBaseURL: false
        ) else { return nil }

        let queryItems = [
            URLQueryItem(name: "response_type", value: "code"),
            URLQueryItem(name: "client_id", value: info.clientID),
            URLQueryItem(name: "redirect_uri", value: redirectURI),
            URLQueryItem(name: "code_challenge", value: challenge(for: verifier)),
            URLQueryItem(name: "code_challenge_method", value: "S256"),
            URLQueryItem(name: "state", value: state),
        ]
        components.queryItems = (components.queryItems ?? []) + queryItems
        return components.url
    }

    static let callbackScheme = "photonic"
    static let redirectURI = "photonic://oauth/callback"

    private static func randomBytes(count: Int) -> Data {
        var bytes = [UInt8](repeating: 0, count: count)
        let status = SecRandomCopyBytes(kSecRandomDefault, count, &bytes)
        if status != errSecSuccess {
            for index in bytes.indices {
                bytes[index] = UInt8.random(in: .min ... .max)
            }
        }
        return Data(bytes)
    }

    private static func base64URL(_ data: Data) -> String {
        data.base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }
}
