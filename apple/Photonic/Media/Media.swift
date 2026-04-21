//
//  Media.swift
//  Photonic
//
//  Created by Max Vissing on 12.01.25.
//

import Foundation

struct Media: Identifiable {
    let id: UUID
    let previewUrl: URL
    let dateTaken: Date?
}
