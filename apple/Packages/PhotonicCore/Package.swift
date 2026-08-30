// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "PhotonicCore",
    platforms: [
        .iOS(.v18),
        .macOS(.v15)
    ],
    products: [
        .library(name: "PhotonicCore", targets: ["PhotonicCore"])
    ],
    targets: [
        .target(name: "PhotonicCore"),
        .testTarget(
            name: "PhotonicCoreTests",
            dependencies: [
                .target(name: "PhotonicCore")
            ]
        )
    ],
    swiftLanguageModes: [.v5]
)
