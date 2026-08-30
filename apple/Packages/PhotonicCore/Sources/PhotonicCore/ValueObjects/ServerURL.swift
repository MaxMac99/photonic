import Foundation

/// A validated base URL of a Photonic server (http/https, host required,
/// no query or fragment). Constructing this type is the boundary validation
/// for every server address the user or a server response provides.
public struct ServerURL: Hashable, Sendable, Codable {
    public let rawValue: URL

    public init?(_ string: String) {
        guard
            let url = URL(string: string),
            let scheme = url.scheme?.lowercased(),
            scheme == "http" || scheme == "https",
            let host = url.host,
            !host.isEmpty,
            url.query == nil,
            url.fragment == nil
        else { return nil }
        self.rawValue = url
    }

    public var absoluteString: String {
        rawValue.absoluteString
    }
}

extension ServerURL: CustomStringConvertible {
    public var description: String {
        absoluteString
    }
}

extension ServerURL {
    public init(from decoder: Decoder) throws {
        let string = try decoder.singleValueContainer().decode(String.self)
        guard let url = ServerURL(string) else {
            throw DecodingError.dataCorruptedError(
                in: try decoder.singleValueContainer(),
                debugDescription: "Invalid server URL: \(string)"
            )
        }
        self.rawValue = url.rawValue
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(absoluteString)
    }
}
