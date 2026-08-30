import Foundation

/// A content checksum used for dedup against the server. Lowercase hex.
public struct MediaHash: Hashable, Sendable, Codable {
    public let value: String

    private static let hexCharacters = Set("0123456789abcdef")

    public init?(_ value: String) {
        let lowered = value.lowercased()
        let isValid = (16 ... 128).contains(lowered.count)
            && lowered.allSatisfy(Self.hexCharacters.contains)
        guard isValid else { return nil }
        self.value = lowered
    }
}

extension MediaHash: CustomStringConvertible {
    public var description: String {
        value
    }
}

public extension MediaHash {
    init(from decoder: Decoder) throws {
        let string = try decoder.singleValueContainer().decode(String.self)
        guard let hash = MediaHash(string) else {
            throw try DecodingError.dataCorruptedError(
                in: decoder.singleValueContainer(),
                debugDescription: "Invalid media hash: \(string)"
            )
        }
        value = hash.value
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(value)
    }
}
