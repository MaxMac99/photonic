import ComposableArchitecture
import PhotonicCore
import SwiftUI

struct AuthView: View {
    let store: StoreOf<AuthFeature>

    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "photo.badge.arrow.down")
                .font(.system(size: 48))
                .foregroundStyle(Color.accentColor)

            Text("Photonic")
                .font(.largeTitle.weight(.semibold))

            Text("Back up your photo library to your own server.")
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            if store.isSigningIn {
                ProgressView("Signing in…")
            } else {
                Button("Sign in") {
                    store.send(.signInTapped)
                }
                .buttonStyle(.borderedProminent)
            }

            if let message = store.errorMessage {
                Text(message)
                    .font(.footnote)
                    .foregroundStyle(.red)
                    .multilineTextAlignment(.center)
            }
        }
        .padding(32)
    }
}

private var previewSession: AuthSession? {
    let serverURL = ServerURL("https://photonic.example.com")
    let info = serverURL.map {
        ServerInfo(
            version: "1.2.3",
            clientID: "photonic-ios",
            tokenURL: $0,
            authorizeURL: $0
        )
    }
    guard
        let serverURL,
        let info,
        let accessToken = AccessToken(value: "preview", expiresAt: .distantFuture),
        let refreshToken = RefreshToken(value: "preview")
    else { return nil }
    return AuthSession(
        serverURL: serverURL,
        serverInfo: info,
        accessToken: accessToken,
        refreshToken: refreshToken
    )
}

#Preview {
    if let session = previewSession {
        AuthView(
            store: Store(initialState: AuthFeature.State()) {
                AuthFeature()
            } withDependencies: {
                $0.authClient.signIn = { _, _ in session }
            }
        )
    }
}
