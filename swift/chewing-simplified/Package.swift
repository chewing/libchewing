// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "ChewingSimplified",
    platforms: [
        // Adjust minimum deployment targets as appropriate
        .iOS(.v16),
    ],
    products: [
        // Public-facing library name: "Chewing"
        .library(
            name: "ChewingSimplified",
            type: .dynamic,
            targets: ["ChewingSimplified"]
        ),
    ],
    targets: [
        // 1) Binary target pointing at the XCFramework you built
        .binaryTarget(
            name: "CLibChewing",
            path: "libchewing.xcframework"
        ),
        
        // 2) Swift wrapper target that depends on the XCFramework and bundles resources
        .target(
            name: "ChewingSimplified",
            dependencies: [
                .target(name: "CLibChewing"),
            ],
            path: "Sources/ChewingSimplified",
            resources: [
                .process("Resources")
            ]
        ),
    ]
)
