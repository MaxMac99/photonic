import Foundation
import Testing

/// Machine-checked architecture rules (R1-R13) from
/// `docs/ios-architecture-rules.md`. Source-scanning via `#filePath` works
/// locally and on macOS CI runners where the checkout exists at compile time.
struct ArchitectureTests {
    private struct Rule {
        let prefix: String
        let allow: Set<String>?
        let deny: Set<String>
    }

    private static let sourceRoot = URL(fileURL: #filePath)
        .deletingLastPathComponent()
        .deletingLastPathComponent()

    /// Most specific prefix wins; rules are matched longest-prefix first.
    private static let rules: [Rule] = [
        Rule(prefix: "Photonic/Features/Auth/Adapters", allow: nil, deny: ["SwiftUI", "UIKit"]),
        Rule(prefix: "Photonic/Features/Backup/Adapters", allow: nil, deny: ["SwiftUI", "UIKit"]),
        Rule(prefix: "Photonic/Features/Library/Adapters", allow: nil, deny: ["SwiftUI", "UIKit"]),
        Rule(prefix: "Photonic/Features/Settings/Adapters", allow: nil, deny: ["SwiftUI", "UIKit"]),
        Rule(
            prefix: "Photonic/Features/Auth/Views",
            allow: ["SwiftUI", "ComposableArchitecture", "PhotonicCore"],
            deny: []
        ),
        Rule(
            prefix: "Photonic/Features/Backup/Views",
            allow: ["SwiftUI", "ComposableArchitecture", "PhotonicCore"],
            deny: []
        ),
        Rule(
            prefix: "Photonic/Features/Library/Views",
            allow: ["SwiftUI", "ComposableArchitecture", "PhotonicCore"],
            deny: []
        ),
        Rule(
            prefix: "Photonic/Features/Settings/Views",
            allow: ["SwiftUI", "ComposableArchitecture", "PhotonicCore"],
            deny: []
        ),
        Rule(
            prefix: "Photonic/Features",
            allow: [
                "Foundation",
                "ComposableArchitecture",
                "Dependencies",
                "DependenciesMacros",
                "PhotonicAPI",
                "PhotonicCore"
            ],
            deny: []
        ),
        Rule(
            prefix: "Photonic/App",
            allow: [
                "Foundation",
                "SwiftUI",
                "ComposableArchitecture",
                "Dependencies",
                "PhotonicAPI",
                "PhotonicCore"
            ],
            deny: []
        ),
        Rule(
            prefix: "Packages/PhotonicAPI",
            allow: nil,
            deny: [
                "SwiftUI",
                "UIKit",
                "SwiftData",
                "Photos",
                "PhotosUI",
                "Security",
                "AuthenticationServices"
            ]
        ),
        Rule(prefix: "Packages/PhotonicCore", allow: ["Foundation"], deny: [])
    ]

    /// Symbols and APIs that may only appear inside adapters (R3).
    private static let adapterOnlySymbols = [
        "UserDefaults",
        "SecItemCopyMatching",
        "SecItemAdd",
        "SecItemDelete",
        "kSecClass",
        "@Model",
        "PHPhotoLibrary",
        "PHAssetCollection",
        "UIApplication.shared"
    ]

    /// Dev-only tooling must never be imported (R7); currently unused.
    private static let forbiddenImports = ["Inject", "XcodebuildNvimPreview"]

    @Test
    func noArchitectureViolations() throws {
        var violations: [String] = []

        for fileURL in try Self.swiftFiles() {
            let relativePath = String(
                fileURL.path
                    .dropFirst(Self.sourceRoot.path.count + 1)
            )
            let contents = try String(contentsOf: fileURL, encoding: .utf8)

            for (lineNumber, line) in contents.enumeratedLines() {
                if let moduleName = Self.importedModule(in: line) {
                    if Self.forbiddenImports.contains(moduleName) {
                        violations.append("\(relativePath):\(lineNumber + 1): import \(moduleName) is forbidden (R7)")
                        continue
                    }
                    if let violation = Self.importViolation(
                        moduleName: moduleName,
                        relativePath: relativePath
                    ) {
                        violations.append("\(relativePath):\(lineNumber + 1): \(violation)")
                    }
                }

                if let symbol = Self.adapterOnlySymbols.first(where: { line.contains($0) }),
                   !relativePath.contains("/Adapters/") {
                    violations.append(
                        "\(relativePath):\(lineNumber + 1): \(symbol) is adapter-only (R3)"
                    )
                }
            }
        }

        #expect(
            violations.isEmpty,
            "Architecture rule violations found:\n\(violations.joined(separator: "\n"))"
        )
    }

    // MARK: - Helpers

    private static func swiftFiles() throws -> [URL] {
        var files: [URL] = []
        let enumerator = FileManager.default.enumerator(
            at: sourceRoot,
            includingPropertiesForKeys: [.isRegularFileKey]
        )
        for case let url as URL in enumerator ?? anyIterator() {
            guard url.pathExtension == "swift" else { continue }
            let path = url.path
            if path.contains("/Photonic/") || path.contains("/Packages/PhotonicAPI/Sources/")
                || path.contains("/Packages/PhotonicCore/Sources/") {
                files.append(url)
            }
        }
        return files.sorted { $0.path < $1.path }
    }

    private static func anyIterator() -> IndexingIterator<[URL]> {
        var empty: [URL] = []
        return empty.makeIterator()
    }

    private static func importedModule(in line: String) -> String? {
        let trimmed = line.trimmingCharacters(in: .whitespaces)
        guard trimmed.hasPrefix("import ") else { return nil }
        let parts = trimmed
            .dropFirst("import ".count)
            .split(separator: " ")
            .map(String.init)
        return parts.first
    }

    private static func importViolation(moduleName: String, relativePath: String) -> String? {
        guard
            let rule = rules
            .sorted(by: { $0.prefix.count > $1.prefix.count })
            .first(where: { relativePath.hasPrefix($0.prefix) })
        else { return nil }

        if rule.deny.contains(moduleName) {
            return "import \(moduleName) is forbidden under \(rule.prefix) (R3)"
        }
        if let allow = rule.allow, !allow.contains(moduleName) {
            return "import \(moduleName) is not allowed under \(rule.prefix) (R1/R3)"
        }
        return nil
    }
}
