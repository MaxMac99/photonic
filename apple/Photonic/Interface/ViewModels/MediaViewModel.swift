//
//  MediaViewModel.swift
//  Photonic
//
//  Interface Layer - Media Browser View Model
//

import Foundation
import SwiftUI

/// View model for the media browsing and management screen
///
/// Manages the display and interaction with media items and albums.
/// Provides functionality for browsing photos, viewing albums, and
/// selecting media for various operations.
@MainActor
final class MediaViewModel: ObservableObject {
    
    // MARK: - Dependencies
    
    private let mediaRepository: MediaRepository
    private let albumRepository: AlbumRepository
    
    // MARK: - Published Properties
    
    /// List of all albums
    @Published var albums: [Album] = []
    
    /// Currently displayed media items
    @Published var mediaItems: [MediaItem] = []
    
    /// Currently selected album for filtering
    @Published var selectedAlbum: Album?
    
    /// Loading state for media fetching
    @Published var isLoadingMedia: Bool = false
    
    /// Loading state for albums
    @Published var isLoadingAlbums: Bool = false
    
    /// Error message for display
    @Published var errorMessage: String?
    
    /// Search query for filtering media
    @Published var searchQuery: String = ""
    
    /// Selected media items for batch operations
    @Published var selectedMediaIds: Set<String> = []
    
    // MARK: - Private Properties
    
    private var loadTask: Task<Void, Never>?
    
    // MARK: - Initialization
    
    init(mediaRepository: MediaRepository, albumRepository: AlbumRepository) {
        self.mediaRepository = mediaRepository
        self.albumRepository = albumRepository
    }
    
    // MARK: - Public Methods
    
    /// Loads all albums from the repository
    func loadAlbums() async {
        isLoadingAlbums = true
        errorMessage = nil
        
        do {
            albums = try await albumRepository.fetchAll()
            isLoadingAlbums = false
        } catch {
            isLoadingAlbums = false
            errorMessage = "Failed to load albums: \(error.localizedDescription)"
        }
    }
    
    /// Loads media items for the selected album
    func loadMedia(for album: Album? = nil) async {
        // Cancel any existing load task
        loadTask?.cancel()
        
        loadTask = Task {
            isLoadingMedia = true
            errorMessage = nil
            mediaItems = []
            
            do {
                let albumsToLoad = album.map { [$0] } ?? albums
                let mediaStream = try await mediaRepository.listMedia(in: albumsToLoad)
                
                var items: [MediaItem] = []
                for try await item in mediaStream {
                    guard !Task.isCancelled else { break }
                    items.append(item)
                    
                    // Update UI periodically during loading
                    if items.count % 50 == 0 {
                        mediaItems = items
                    }
                }
                
                if !Task.isCancelled {
                    mediaItems = items
                }
                isLoadingMedia = false
            } catch {
                if !Task.isCancelled {
                    isLoadingMedia = false
                    errorMessage = "Failed to load media: \(error.localizedDescription)"
                }
            }
        }
    }
    
    /// Refreshes both albums and media
    func refresh() async {
        await loadAlbums()
        await loadMedia(for: selectedAlbum)
    }
    
    /// Selects an album and loads its media
    func selectAlbum(_ album: Album?) async {
        selectedAlbum = album
        await loadMedia(for: album)
    }
    
    /// Toggles selection of a media item
    func toggleMediaSelection(_ mediaId: String) {
        if selectedMediaIds.contains(mediaId) {
            selectedMediaIds.remove(mediaId)
        } else {
            selectedMediaIds.insert(mediaId)
        }
    }
    
    /// Selects all visible media items
    func selectAllMedia() {
        selectedMediaIds = Set(mediaItems.map { $0.id })
    }
    
    /// Deselects all media items
    func deselectAllMedia() {
        selectedMediaIds.removeAll()
    }
    
    /// Returns filtered media items based on search query
    var filteredMediaItems: [MediaItem] {
        guard !searchQuery.isEmpty else { return mediaItems }
        
        return mediaItems.filter { item in
            // Filter based on date, location, or other metadata
            // This is a simple implementation - enhance as needed
            return true
        }
    }
}