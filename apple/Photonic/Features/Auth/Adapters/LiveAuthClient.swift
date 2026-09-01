import AuthenticationServices
import Dependencies
import Foundation
import PhotonicCore

/// Runs the OAuth2 authorization-code flow with PKCE in
/// `ASWebAuthenticationSession`, exchanges the code at `token_url`, and
/// persists the resulting session in the Keychain.
extension AuthClient: DependencyKey {
    static var liveValue: AuthClient {
        AuthClient(
            restoreSession: { AuthSessionStore.load() },
            signIn: { serverURL, info in
                let verifier = PKCE.makeVerifier()
                let state = PKCE.makeState()

                guard let authorizeURL = PKCE.authorizeURL(
                    info: info,
                    verifier: verifier,
                    redirectURI: PKCE.redirectURI,
                    state: state
                ) else {
                    throw AuthError.invalidAuthorizeURL
                }

                let callback = try await WebAuthentication.authenticate(
                    url: authorizeURL,
                    callbackScheme: PKCE.callbackScheme
                )
                let code = try Self.authorizationCode(from: callback, expectedState: state)

                let tokens = try await TokenExchange.exchange(
                    info: info,
                    code: code,
                    verifier: verifier
                )
                let session = AuthSession(
                    serverURL: serverURL,
                    serverInfo: info,
                    accessToken: tokens.access,
                    refreshToken: tokens.refresh
                )
                try AuthSessionStore.save(session)
                return session
            },
            signOut: { AuthSessionStore.delete() }
        )
    }

    private static func authorizationCode(
        from callback: URL,
        expectedState: String
    ) throws -> String {
        guard
            let components = URLComponents(url: callback, resolvingAgainstBaseURL: false),
            components.host == "oauth",
            let queryItems = components.queryItems,
            queryItems.first(where: { $0.name == "state" })?.value == expectedState,
            let code = queryItems.first(where: { $0.name == "code" })?.value,
            !code.isEmpty
        else { throw AuthError.invalidCallback }
        return code
    }
}

/// Presents `ASWebAuthenticationSession` and resumes with the callback URL.
enum WebAuthentication {
    final class Anchor: NSObject, ASWebAuthenticationPresentationContextProviding {
        func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
            UIApplication.shared.connectedScenes
                .compactMap { ($0 as? UIWindowScene)?.keyWindow }
                .first ?? ASPresentationAnchor()
        }
    }

    static func authenticate(url: URL, callbackScheme: String) async throws -> URL {
        try await withCheckedThrowingContinuation { continuation in
            let anchor = Anchor()
            let session = ASWebAuthenticationSession(
                url: url,
                callbackURLScheme: callbackScheme
            ) { callbackURL, error in
                if let callbackURL {
                    continuation.resume(returning: callbackURL)
                } else {
                    continuation.resume(throwing: error ?? AuthError.invalidCallback)
                }
            }
            session.presentationContextProvider = anchor
            session.start()
        }
    }
}

/// Exchanges the authorization code for tokens at the server's `token_url`.
enum TokenExchange {
    struct Tokens {
        let access: AccessToken
        let refresh: RefreshToken
    }

    private struct Response: Codable {
        let accessToken: String?
        let refreshToken: String?
        let expiresIn: Double?

        enum CodingKeys: String, CodingKey {
            case accessToken = "access_token"
            case refreshToken = "refresh_token"
            case expiresIn = "expires_in"
        }
    }

    static func exchange(info: ServerInfo, code: String, verifier: String) async throws -> Tokens {
        guard
            let tokenURL = info.tokenURL,
            let clientID = info.clientID
        else { throw AuthError.tokenExchangeFailed }

        var request = URLRequest(url: tokenURL.rawValue)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
        request.httpBody = formBody([
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": PKCE.redirectURI,
            "client_id": clientID,
            "code_verifier": verifier
        ])

        let (data, response) = try await URLSession.shared.data(for: request)
        guard (response as? HTTPURLResponse)?.statusCode == 200 else {
            throw AuthError.tokenExchangeFailed
        }

        let decoded = try JSONDecoder().decode(Response.self, from: data)
        guard
            let rawAccess = decoded.accessToken,
            let rawRefresh = decoded.refreshToken,
            let expiresIn = decoded.expiresIn,
            let access = AccessToken(
                value: rawAccess,
                expiresAt: Date().addingTimeInterval(expiresIn)
            ),
            let refresh = RefreshToken(value: rawRefresh)
        else { throw AuthError.invalidTokenResponse }

        return Tokens(access: access, refresh: refresh)
    }

    private static func formBody(_ parameters: [String: String]) -> Data {
        let encoded = parameters
            .map { key, value in
                "\(urlEscaped(key))=\(urlEscaped(value))"
            }
            .joined(separator: "&")
        return Data(encoded.utf8)
    }

    private static func urlEscaped(_ string: String) -> String {
        var allowed = CharacterSet.alphanumerics
        allowed.insert(charactersIn: "-._~")
        return string.addingPercentEncoding(withAllowedCharacters: allowed) ?? string
    }
}
