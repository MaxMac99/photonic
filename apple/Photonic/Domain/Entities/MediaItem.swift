//
//  MediaItem.swift
//  Photonic
//
//  Domain Entity
//

import Foundation

public struct MediaItem: Equatable, Hashable {
    public let id: String
    public let checksum: String
    public let createdAt: Date
    public let mimeType: String
    public let sizeInBytes: Int64
    public let width: Int?
    public let height: Int?
    public let duration: TimeInterval?
    public let location: Location?
    public let albumIds: Set<String>
    
    public struct Location: Equatable, Hashable {
        public let latitude: Double
        public let longitude: Double
        public let altitude: Double?
        
        public init(latitude: Double, longitude: Double, altitude: Double? = nil) {
            self.latitude = latitude
            self.longitude = longitude
            self.altitude = altitude
        }
    }
    
    public init(id: String, checksum: String, createdAt: Date, mimeType: String = "image/jpeg", sizeInBytes: Int64 = 0, width: Int? = nil, height: Int? = nil, duration: TimeInterval? = nil, location: Location? = nil, albumIds: Set<String> = []) {
        self.id = id
        self.checksum = checksum
        self.createdAt = createdAt
        self.mimeType = mimeType
        self.sizeInBytes = sizeInBytes
        self.width = width
        self.height = height
        self.duration = duration
        self.location = location
        self.albumIds = albumIds
    }
}

extension MediaItem {
    var isVideo: Bool {
        mimeType.hasPrefix("video/")
    }
    
    var isImage: Bool {
        mimeType.hasPrefix("image/")
    }
    
    var aspectRatio: Double? {
        guard let width = width, let height = height, height > 0 else { return nil }
        return Double(width) / Double(height)
    }
}