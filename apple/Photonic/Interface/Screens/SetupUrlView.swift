//
//  SetupUrlView.swift
//  Photonic
//
//  Interface Layer - Setup Screen
//

import SwiftUI

#if DEBUG
    import XcodebuildNvimPreview
#endif

struct SetupUrlView: View {

    @StateObject private var viewModel: SetupViewModel
    @FocusState private var isUrlFieldFocused: Bool
    @Environment(\.dismiss) private var dismiss
    private let onSetupComplete: ((ServerConfiguration) -> Void)?

    init(
        viewModel: @autoclosure @escaping () -> SetupViewModel,
        onSetupComplete: ((ServerConfiguration) -> Void)? = nil
    ) {
        self._viewModel = StateObject(wrappedValue: viewModel())
        self.onSetupComplete = onSetupComplete
    }

    var body: some View {
        VStack(spacing: 24) {
            headerSection

            Form {
                urlInputSection
                actionSection
                errorSection
            }
            .formStyle(.grouped)
        }
        .frame(maxWidth: 650)
        .navigationTitle("")
        .navigationBarHidden(true)
        .onAppear {
            isUrlFieldFocused = true
        }
        .onChange(of: viewModel.setupState) { _, newState in
            if case .success(let config) = newState {
                onSetupComplete?(config)
                dismiss()
            }
        }
        #if DEBUG
            .setupNvimPreview {
                SetupUrlView(viewModel: MockSetupViewModel())
            }
        #endif
    }

    // MARK: - View Components

    private var headerSection: some View {
        VStack(spacing: 8) {
            Image(systemName: "photo.on.rectangle.angled")
                .font(.system(size: 64))
                .foregroundStyle(.tint)

            Text("Photonic")
                .font(.largeTitle)
                .fontWeight(.bold)

            Text("Connect to your Photonic server")
                .foregroundStyle(.secondary)
        }
        .padding(.top, 32)
    }

    private var urlInputSection: some View {
        Section {
            HStack {
                Image(systemName: "server.rack")
                    .foregroundStyle(.secondary)

                TextField(
                    "Server URL",
                    text: $viewModel.serverUrlString,
                    prompt: Text(makePlainText("https://photonic.example.com"))
                )
                .keyboardType(.URL)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .focused($isUrlFieldFocused)
                .disabled(viewModel.isConnecting)
                .onSubmit {
                    Task {
                        await viewModel.connectToServer()
                    }
                }

                if viewModel.validateUrl() {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(.green)
                }
            }
        } header: {
            Text("Server Configuration")
        } footer: {
            Text(
                "Enter the URL of your Photonic server. If no scheme is provided, HTTPS will be used."
            )
            .font(.caption)
            .foregroundStyle(.secondary)
        }
    }

    @ViewBuilder
    private var errorSection: some View {
        if let errorMessage = viewModel.errorMessage {
            Section {
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundStyle(.red)

                    Text(errorMessage)
                        .foregroundStyle(.red)
                }
            }
        }
    }

    private var actionSection: some View {
        Section {
            Button(action: {
                Task {
                    await viewModel.connectToServer()
                }
            }) {
                HStack {
                    Text(buttonTitle)
                        .fontWeight(.medium)

                    Spacer()

                    if viewModel.isConnecting {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle())
                            .scaleEffect(0.8)
                    } else {
                        Image(systemName: "arrow.right.circle.fill")
                    }
                }
            }
            .disabled(!viewModel.validateUrl() || viewModel.isConnecting)
        }
    }

    private var buttonTitle: String {
        switch viewModel.setupState {
        case .idle, .error:
            return "Connect"
        case .connecting:
            return "Connecting..."
        case .authenticating:
            return "Authenticating..."
        case .verifying:
            return "Verifying..."
        case .success:
            return "Connected"
        }
    }

    private func makePlainText(_ url: String) -> AttributedString {
        var attr = AttributedString(url)
        attr.link = nil
        attr.foregroundColor = Color(.placeholderText)
        return attr
    }
}

// MARK: - Mock ViewModel for Previews

@MainActor
final class MockSetupViewModel: SetupViewModel {
    init(state: SetupState = .idle, serverUrl: String = "") {
        super.init(discoverServerUseCase: MockDiscoverServerUseCase())
        self.setupState = state
        self.serverUrlString = serverUrl
    }
}

// MARK: - Mock Use Case

final class MockDiscoverServerUseCase: DiscoverServerUseCaseProtocol {
    var shouldSucceed = true
    var delay: TimeInterval = 1.0

    func discoverAndConnect(urlString: String) async throws -> ServerConfiguration {
        try await Task.sleep(nanoseconds: UInt64(delay * 1_000_000_000))

        if shouldSucceed {
            return ServerConfiguration(
                serverUrl: URL(string: "https://photonic.example.com")!,
                clientId: "mock-client-id",
                tokenUrl: URL(string: "https://photonic.example.com/oauth/token")!,
                authorizationUrl: URL(string: "https://photonic.example.com/oauth/authorize")!
            )!
        } else {
            throw DomainError.networkError("Mock connection failed")
        }
    }

    func validateAndNormalizeUrl(_ urlString: String) -> URL? {
        guard !urlString.isEmpty else { return nil }
        return URL(string: urlString.contains("://") ? urlString : "https://\(urlString)")
    }
}

// MARK: - Previews

#Preview("Default") {
    SetupUrlView(viewModel: MockSetupViewModel())
        .preferredColorScheme(.dark)
}

#Preview("With URL") {
    SetupUrlView(
        viewModel: MockSetupViewModel(
            state: .idle,
            serverUrl: "https://photonic.example.de"
        ))
}

#Preview("Connecting") {
    SetupUrlView(viewModel: MockSetupViewModel(state: .connecting))
}

#Preview("Error State") {
    SetupUrlView(
        viewModel: {
            let vm = MockSetupViewModel(
                state: .error("Failed to connect to server"),
                serverUrl: "https://photonic.example.de")
            vm.errorMessage = "Failed to connect to server"
            return vm
        }())
}
