//
//  SettingsView.swift
//  Photonic
//
//  Interface Layer - Settings Screen
//

import SwiftUI
import SwiftData

#if DEBUG
    import XcodebuildNvimPreview
#endif

struct SettingsView: View {
    
    private let logger = LoggerFactory.logger(for: .ui)
    @Environment(\.apiClient) private var client
    @Environment(\.compositionRoot) private var compositionRoot
    @Environment(\.modelContext) private var modelContext
    @ObservedObject var viewModel: SettingsViewModel
    @State private var showingSignOutAlert = false

    var body: some View {
        NavigationStack {
            List {
                accountSection
                serverSection
                backupSection
                aboutSection
            }
            .navigationTitle("Settings")
            .alert("Sign Out", isPresented: $showingSignOutAlert) {
                Button("Cancel", role: .cancel) {}
                Button("Sign Out", role: .destructive) {
                    Task {
                        await signOut()
                    }
                }
            } message: {
                Text(
                    "Are you sure you want to sign out? You will need to sign in again to access your photos."
                )
            }
            .refreshable {
                await viewModel.loadData()
            }
        }
        .task {
            await viewModel.loadData()
        }
        #if DEBUG
            .setupNvimPreview {
                SettingsView(viewModel: SettingsViewModel(
                    authRepository: PreviewDependencies.mockAuthRepository,
                    userRepository: PreviewDependencies.mockUserRepository,
                    serverConfigurationRepository: PreviewDependencies.mockServerConfigRepository
                ))
                .environment(\.apiClient, PreviewDependencies.createMockClient())
                .environment(\.compositionRoot, PreviewDependencies.createMockCompositionRoot())
            }
        #endif
    }

    // MARK: - Sections

    private var accountSection: some View {
        Section("Account") {
            HStack {
                Label("User", systemImage: "person.circle")
                Spacer()
                if viewModel.isLoadingUser {
                    ProgressView()
                        .scaleEffect(0.8)
                } else {
                    Text(viewModel.userEmail)
                        .foregroundStyle(.secondary)
                }
            }

            HStack {
                Label("Storage", systemImage: "internaldrive")
                Spacer()
                VStack(alignment: .trailing) {
                    Text(viewModel.storageUsageText)
                        .foregroundStyle(.secondary)
                    ProgressView(value: viewModel.storageUsagePercentage)
                        .frame(width: 100)
                }
            }

            HStack {
                Label("Photos", systemImage: "photo")
                Spacer()
                Text(viewModel.photoCount)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var serverSection: some View {
        Section("Server") {
            HStack {
                Label("Server URL", systemImage: "server.rack")
                Spacer()
                Text(viewModel.serverUrl)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }

            HStack {
                Label("Version", systemImage: "info.circle")
                Spacer()
                Text(viewModel.serverVersion)
                    .foregroundStyle(.secondary)
            }

            HStack {
                Label("Connection", systemImage: "wifi")
                Spacer()
                HStack(spacing: 4) {
                    Circle()
                        .fill(viewModel.isConnected ? .green : .red)
                        .frame(width: 8, height: 8)
                    Text(viewModel.isConnected ? "Connected" : "Disconnected")
                        .foregroundStyle(.secondary)
                }
            }

            Button(role: .destructive) {
                showingSignOutAlert = true
            } label: {
                Label("Sign Out", systemImage: "rectangle.portrait.and.arrow.forward")
                    .foregroundStyle(.red)
            }
        }
    }

    private var backupSection: some View {
        Section("Backup") {
            if let compositionRoot = compositionRoot {
                NavigationLink(destination: BackupAlbumSelectionView(
                    viewModel: compositionRoot.makeBackupAlbumSelectionViewModel(modelContext: modelContext)
                )) {
                    Label("Album Selection", systemImage: "photo.on.rectangle.angled")
                }
            }
            
            Toggle(isOn: $viewModel.autoBackupEnabled) {
                Label("Auto Backup", systemImage: "arrow.clockwise")
            }

            Toggle(isOn: $viewModel.backupOverCellularEnabled) {
                Label("Backup Over Cellular", systemImage: "antenna.radiowaves.left.and.right")
            }

            HStack {
                Label("Last Backup", systemImage: "clock")
                Spacer()
                Text(viewModel.lastBackupText)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var aboutSection: some View {
        Section("About") {
            HStack {
                Label("App Version", systemImage: "app.badge")
                Spacer()
                Text(viewModel.appVersion)
                    .foregroundStyle(.secondary)
            }

            Link(destination: URL(string: "https://github.com/photonic/photonic-ios")!) {
                HStack {
                    Label("GitHub", systemImage: "link")
                    Spacer()
                    Image(systemName: "arrow.up.right.square")
                        .foregroundStyle(.secondary)
                }
            }

            Link(destination: URL(string: "https://photonic.app/privacy")!) {
                HStack {
                    Label("Privacy Policy", systemImage: "hand.raised")
                    Spacer()
                    Image(systemName: "arrow.up.right.square")
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    // MARK: - Actions

    private func signOut() async {
        do {
            try await viewModel.signOut()
        } catch {
            // Handle error - could show an alert
            logger.error("Sign out failed", error: error)
        }
    }
}

// MARK: - Previews

#Preview("Settings") {
    SettingsView(viewModel: SettingsViewModel(
        authRepository: PreviewDependencies.mockAuthRepository,
        userRepository: PreviewDependencies.mockUserRepository,
        serverConfigurationRepository: PreviewDependencies.mockServerConfigRepository
    ))
    .environment(\.apiClient, PreviewDependencies.createMockClient())
    .environment(\.compositionRoot, PreviewDependencies.createMockCompositionRoot())
}

#Preview("Settings - Dark Mode") {
    SettingsView(viewModel: SettingsViewModel(
        authRepository: PreviewDependencies.mockAuthRepository,
        userRepository: PreviewDependencies.mockUserRepository,
        serverConfigurationRepository: PreviewDependencies.mockServerConfigRepository
    ))
    .environment(\.apiClient, PreviewDependencies.createMockClient())
    .environment(\.compositionRoot, PreviewDependencies.createMockCompositionRoot())
    .preferredColorScheme(.dark)
}
