//
//  MediaHash.swift
//  Photonic
//
//  Domain Value Object
//

import Foundation

struct MediaHash: Equatable, Hashable {
    let value: String
    let algorithm: HashAlgorithm

    enum HashAlgorithm: String, Equatable {
        case sha256
        case sha512
        case md5
    }

    init?(value: String, algorithm: HashAlgorithm = .sha256) {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()

        // Validate hash format based on algorithm
        let isValid: Bool = switch algorithm {
        case .sha256:
            trimmed.count == 64 && trimmed.allSatisfy(\.isHexDigit)
        case .sha512:
            trimmed.count == 128 && trimmed.allSatisfy(\.isHexDigit)
        case .md5:
            trimmed.count == 32 && trimmed.allSatisfy(\.isHexDigit)
        }

        guard isValid else { return nil }

        self.value = trimmed
        self.algorithm = algorithm
    }
}

extension Character {
    var isHexDigit: Bool {
        ("0" ... "9").contains(self) || ("a" ... "f").contains(self)
    }
}
