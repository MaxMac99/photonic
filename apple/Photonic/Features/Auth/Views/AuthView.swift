import ComposableArchitecture
import PhotonicCore
import SwiftUI

struct AuthView: View {
    @Bindable var store: StoreOf<AuthFeature>

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

            if let info = store.connectedServer {
                connectedSection(info)
            } else {
                connectSection
            }

            if let message = store.errorMessage {
                Text(message)
                    .font(.footnote)
                    .foregroundStyle(.red)
                    .multilineTextAlignment(.center)
            }
        }
        .padding(32)
        .onAppear {
            store.send(.onAppear)
        }
    }

    private func connectedSection(_ info: ServerInfo) -> some View {
        VStack(spacing: 12) {
            LabeledContent("Server", value: info.authorizeURL.rawValue.host() ?? "")
            LabeledContent("Version", value: info.version)

            if store.isSigningIn {
                ProgressView("Signing in…")
            } else {
                Button("Sign in") {
                    store.send(.signInTapped)
                }
                .buttonStyle(.borderedProminent)
            }
        }
    }

    private var connectSection: some View {
        VStack(spacing: 12) {
            TextField("https://photonic.example.com", text: $store.serverURLText)
                .keyboardType(.URL)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .textFieldStyle(.roundedBorder)

            Button(store.isConnecting ? "Connecting…" : "Connect") {
                store.send(.connectTapped)
            }
            .buttonStyle(.borderedProminent)
            .disabled(store.isConnecting || store.serverURLText.isEmpty)

            if let status = store.connectionStatus {
                Text(status)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
        }
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

#Preview("Connect") {
    AuthView(
        store: Store(initialState: AuthFeature.State()) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient = .inMemoryValue
        }
    )
}

#Preview("Signed out, server known") {
    AuthView(
        store: Store(initialState: AuthFeature.State()) {
            AuthFeature()
        } withDependencies: {
            $0.serverConfigurationClient = .inMemoryValue
            $0.discoveryClient.fetchSystemInfo = { serverURL in
                ServerInfo(
                    version: "1.2.3",
                    clientID: "photonic-ios",
                    tokenURL: serverURL,
                    authorizeURL: serverURL
                )
            }
        }
    )
}
