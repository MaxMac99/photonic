//
//  MainViewModel.swift
//  Photonic
//
//  Interface Layer - Main Screen View Model
//

import Foundation
import SwiftUI

/// View model for the main application screen
///
/// Coordinates the primary application flow and manages high-level
/// navigation state. Acts as the main orchestrator for backup operations
/// and provides access to the backup media use case.
@MainActor
final class MainViewModel: ObservableObject {
    
    private let logger = LoggerFactory.logger(for: .ui)
    
    // MARK: - Dependencies
    
    private let backupService: BackupServiceProtocol
    
    // MARK: - Published Properties
    
    /// Current navigation selection
    @Published var selectedTab: Int = 0
    
    /// Overall backup status
    @Published var isBackupActive: Bool = false
    
    /// Error state for displaying alerts
    @Published var errorMessage: String?
    
    // MARK: - Initialization
    
    init(backupService: BackupServiceProtocol) {
        self.backupService = backupService
    }
    
    // MARK: - Public Methods
    
    /// Initiates a backup for the selected album selections
    func startBackup(selections: [BackupAlbumSelectionEntity]) async {
        isBackupActive = true
        errorMessage = nil
        
        do {
            let progressStream = try await backupService.startBackup(for: selections)
            
            for try await progress in progressStream {
                // Handle progress updates
                // This could be exposed via @Published properties for UI binding
                logger.info("Backup progress: \(progress.processedItems)/\(progress.totalItems)")
            }
            
            isBackupActive = false
        } catch {
            isBackupActive = false
            errorMessage = error.localizedDescription
        }
    }
    
    /// Pauses the current backup operation
    func pauseBackup() async {
        await backupService.pauseBackup()
    }
    
    /// Resumes a paused backup operation
    func resumeBackup() async {
        await backupService.resumeBackup()
    }
    
    /// Cancels the current backup operation
    func cancelBackup() async {
        await backupService.cancelBackup()
        isBackupActive = false
    }
}