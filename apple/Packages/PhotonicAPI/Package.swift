// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "PhotonicAPI",
    platforms: [
        .iOS(.v18),
        .macOS(.v15),
    ],
    products: [
        .library(name: "PhotonicAPI", targets: ["PhotonicAPI"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-openapi-runtime.git", from: "1.7.0"),
        .package(url: "https://github.com/apple/swift-openapi-urlsession.git", from: "1.0.2"),
        .package(url: "https://github.com/apple/swift-openapi-generator.git", from: "1.6.0"),
        .package(path: "../PhotonicCore"),
    ],
    targets: [
        .target(
            name: "PhotonicAPI",
            dependencies: [
                .product(name: "OpenAPIRuntime", package: "swift-openapi-runtime"),
                .product(name: "OpenAPIURLSession", package: "swift-openapi-urlsession"),
                .product(name: "PhotonicCore", package: "PhotonicCore"),
            ],
            plugins: [
                .plugin(name: "OpenAPIGenerator", package: "swift-openapi-generator")
            ]
        ),
        .testTarget(
            name: "PhotonicAPITests",
            dependencies: [
                .target(name: "PhotonicAPI"),
                .product(name: "PhotonicCore", package: "PhotonicCore"),
            ]
        ),
    ],
    swiftLanguageModes: [.v5]
)
