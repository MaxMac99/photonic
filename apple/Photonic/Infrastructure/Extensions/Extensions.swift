//
//  Extensions.swift
//  Photonic
//
//  Created by Max Vissing on 15.03.25.
//

import Foundation
import OSLog

extension Array: Swift.RawRepresentable where Element: Codable {

    private static var logger: Logger {
        LoggerFactory.logger(for: .general)
    }
    public init?(rawValue: String) {
        guard let data = rawValue.data(using: .utf8) else { return nil }
        do {
            self = try JSONDecoder().decode([Element].self, from: data)
        } catch {
            Self.logger.error("Failed to decode array", error: error)
            return nil
        }
    }

    public var rawValue: String {
        guard let data = try? JSONEncoder().encode(self),
            let result = String(data: data, encoding: .utf8)
        else {
            return "[]"
        }

        return result
    }
}

extension Set: Swift.RawRepresentable where Element: Codable {

    private static var logger: Logger {
        LoggerFactory.logger(for: .general)
    }
    public init?(rawValue: String) {
        guard let data = rawValue.data(using: .utf8) else { return nil }
        do {
            self = try JSONDecoder().decode(Set<Element>.self, from: data)
        } catch {
            Self.logger.error("Failed to decode array", error: error)
            return nil
        }
    }

    public var rawValue: String {
        guard let data = try? JSONEncoder().encode(self),
            let result = String(data: data, encoding: .utf8)
        else {
            return "[]"
        }

        return result
    }
}
