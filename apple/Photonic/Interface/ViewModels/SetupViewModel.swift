//
//  SetupViewModel.swift
//  Photonic
//
//  Interface Layer - ViewModel
//

import Foundation
import SwiftUI

@MainActor
class SetupViewModel: ObservableObject {
    // MARK: - Published Properties

    @Published var serverUrlString = ""
    @Published var errorMessage: String?
    @Published var isConnecting = false
    @Published var setupState: SetupState = .idle

    // MARK: - Private Properties

    private let discoverServerUseCase: DiscoverServerUseCaseProtocol

    // MARK: - Types

    enum SetupState: Equatable {
        case idle
        case connecting
        case authenticating
        case verifying
        case success(ServerConfiguration)
        case error(String)
    }

    // MARK: - Initialization

    init(discoverServerUseCase: DiscoverServerUseCaseProtocol) {
        self.discoverServerUseCase = discoverServerUseCase
    }

    // MARK: - Public Methods

    func connectToServer() async {
        guard !serverUrlString.isEmpty else {
            errorMessage = "Please enter a server URL"
            setupState = .error("Please enter a server URL")
            return
        }

        isConnecting = true
        errorMessage = nil
        setupState = .connecting

        do {
            // The use case will handle: discover -> auth -> validate with user_stats
            setupState = .authenticating
            let configuration = try await discoverServerUseCase.discoverAndConnect(
                urlString: serverUrlString
            )

            setupState = .verifying
            // Small delay to show verification state
            try await Task.sleep(nanoseconds: 500_000_000)

            setupState = .success(configuration)
            errorMessage = nil
        } catch let DomainError.validationError(message) {
            errorMessage = message
            setupState = .error(message)
        } catch let DomainError.networkError(message) {
            errorMessage = "Network error: \(message)"
            setupState = .error(errorMessage!)
        } catch let DomainError.serverError(_, message) {
            errorMessage = "Server error: \(message)"
            setupState = .error(errorMessage!)
        } catch AuthError.signInCancelled {
            errorMessage = "Sign in was cancelled"
            setupState = .idle
        } catch let AuthError.signInFailed(reason) {
            errorMessage = "Sign in failed: \(reason)"
            setupState = .error(errorMessage!)
        } catch {
            errorMessage = "Failed to connect: \(error.localizedDescription)"
            setupState = .error(errorMessage!)
        }

        isConnecting = false
    }

    func validateUrl() -> Bool {
        guard !serverUrlString.isEmpty else { return false }
        return discoverServerUseCase.validateAndNormalizeUrl(serverUrlString) != nil
    }

    func normalizedUrl() -> URL? {
        discoverServerUseCase.validateAndNormalizeUrl(serverUrlString)
    }
}
