//
//  BackupSelection.swift
//  Photonic
//
//  Created by Max Vissing on 08.02.25.
//

import Foundation

public struct BackupSelection: Equatable {
    public let assetIdentifier: String
    public let selectedAt: Date
    
    public init(assetIdentifier: String, selectedAt: Date = Date()) {
        self.assetIdentifier = assetIdentifier
        self.selectedAt = selectedAt
    }
}