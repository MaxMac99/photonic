import ComposableArchitecture
import PhotonicCore
import SwiftUI

struct SettingsView: View {
    @Bindable var store: StoreOf<SettingsFeature>

    var body: some View {
        Form {
            Section("Server") {
                TextField("https://photonic.example.com", text: $store.serverURLText)
                    .keyboardType(.URL)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                Button(store.isConnecting ? "Connecting…" : "Connect") {
                    store.send(.connectTapped)
                }
                .disabled(store.isConnecting || store.serverURLText.isEmpty)
            }

            if let info = store.serverInfo {
                Section("Connection") {
                    LabeledContent("Version", value: info.version)
                    LabeledContent("Authorize URL", value: info.authorizeURL?.absoluteString ?? "")
                }
            }

            if let message = store.statusMessage {
                Section {
                    Text(message)
                        .font(.footnote)
                        .foregroundStyle(store.serverInfo == nil ? Color.red : Color.green)
                }
            }
        }
        .onAppear {
            guard !AppRuntime.isRunningUnitTests else { return }
            store.send(.onAppear)
        }
    }
}

#Preview {
    SettingsView(
        store: Store(initialState: SettingsFeature.State()) {
            SettingsFeature()
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
