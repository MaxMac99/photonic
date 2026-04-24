//
//  LoggerFactory.swift
//  Photonic
//
//  Infrastructure - Centralized Logging
//

import Foundation
import OSLog

/// Centralized logger factory for consistent logging across the application
enum LoggerFactory {
    /// Main subsystem identifier for the app
    private static let subsystem = Bundle.main.bundleIdentifier ?? "com.photonic.app"

    /// Predefined logging categories
    enum Category: String {
        case auth = "Auth"
        case api = "API"
        case persistence = "Persistence"
        case photos = "Photos"
        case backup = "Backup"
        case network = "Network"
        case ui = "UI"
        case domain = "Domain"
        case application = "Application"
        case general = "General"
    }

    /// Create a logger for a specific category
    static func logger(for category: Category) -> Logger {
        Logger(subsystem: subsystem, category: category.rawValue)
    }

    /// Create a logger for a specific type (uses type name as category)
    static func logger(for type: (some Any).Type) -> Logger {
        Logger(subsystem: subsystem, category: String(describing: type))
    }
}

/// Extension to make logging more convenient
extension Logger {
    /// Log an error with additional context
    func error(_ message: String, error: Error? = nil, file: String = #file, function: String = #function, line: Int = #line) {
        let logMessage: String
        if let error {
            logMessage = "\(message) - Error: \(error.localizedDescription)"
            self
                .error(
                    "\(message, privacy: .public) - Error: \(error.localizedDescription, privacy: .public) [File: \(file, privacy: .public), Function: \(function, privacy: .public), Line: \(line, privacy: .public)]"
                )
        } else {
            logMessage = message
            self
                .error(
                    "\(message, privacy: .public) [File: \(file, privacy: .public), Function: \(function, privacy: .public), Line: \(line, privacy: .public)]"
                )
        }
        #if DEBUG
        let fileName = (file as NSString).lastPathComponent
        print("🔴 ERROR [\(fileName):\(line)] \(logMessage)")
        #endif
    }

    /// Log a warning with additional context
    func warning(_ message: String, file: String = #file, function: String = #function, line: Int = #line) {
        warning(
            "\(message, privacy: .public) [File: \(file, privacy: .public), Function: \(function, privacy: .public), Line: \(line, privacy: .public)]"
        )
        #if DEBUG
        let fileName = (file as NSString).lastPathComponent
        print("🟡 WARN  [\(fileName):\(line)] \(message)")
        #endif
    }

    /// Log info message
    func info(_ message: String, file: String = #file, function: String = #function, line: Int = #line) {
        info("\(message, privacy: .public)")
        #if DEBUG
        let fileName = (file as NSString).lastPathComponent
        print("🔵 INFO  [\(fileName):\(line)] \(message)")
        #endif
    }

    /// Log debug information (only in DEBUG builds)
    func debug(_ message: String, file: String = #file, function: String = #function, line: Int = #line) {
        #if DEBUG
        debug(
            "\(message, privacy: .public) [File: \(file, privacy: .public), Function: \(function, privacy: .public), Line: \(line, privacy: .public)]"
        )
        let fileName = (file as NSString).lastPathComponent
        print("🟢 DEBUG [\(fileName):\(line)] \(message)")
        #endif
    }

    /// Log verbose/trace information (only in DEBUG builds)
    func verbose(_ message: String, file: String = #file, function: String = #function, line: Int = #line) {
        #if DEBUG
        trace(
            "\(message, privacy: .public) [File: \(file, privacy: .public), Function: \(function, privacy: .public), Line: \(line, privacy: .public)]"
        )
        let fileName = (file as NSString).lastPathComponent
        print("⚪ TRACE [\(fileName):\(line)] \(message)")
        #endif
    }

    /// Log sensitive information with privacy protection
    func logSensitive(_ message: String, sensitiveData: String? = nil) {
        if let sensitiveData {
            info("Operation: \(message, privacy: .public) - Data: \(sensitiveData, privacy: .sensitive(mask: .hash))")
        } else {
            info("\(message, privacy: .public)")
        }
    }
}
