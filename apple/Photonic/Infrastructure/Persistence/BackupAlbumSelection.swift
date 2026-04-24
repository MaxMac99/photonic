//
//  BackupAlbumSelection.swift
//  Photonic
//
//  Created by Max Vissing on 16.03.25.
//

import OSLog
import PhotosUI
import SwiftData
import SwiftUI

@Model
final class BackupAlbumSelection: Hashable {
    var localIdentifier: String
    var name: String
    var selectionType: SelectionType
    var creationDate: Date

    init(localIdentifier: String, name: String, selectionType: SelectionType, creationDate: Date) {
        self.localIdentifier = localIdentifier
        self.name = name
        self.selectionType = selectionType
        self.creationDate = creationDate
    }

    convenience init(collection: PHAssetCollection, selectionType: SelectionType) {
        self.init(
            localIdentifier: collection.localIdentifier,
            name: collection.localizedTitle ?? "",
            selectionType: selectionType,
            creationDate: .now
        )
    }

    enum SelectionType: Codable {
        case included, excluded
    }
}

extension BackupAlbumSelection: Identifiable {
    var id: String {
        localIdentifier
    }
}
