// swift-tools-version:5.9
import PackageDescription

let version = "1.0.1"
let checksum = "6bb037a2e242e87397be298f372d20d61f4a6ee109e2e68a0d7c19fd9af0905f"

let package = Package(
    name: "Chewing",
    platforms: [
        .iOS(.v16),
    ],
    products: [
        .library(
            name: "Chewing",
            type: .dynamic,
            targets: ["Chewing"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-docc-plugin", from: "1.0.0"),
    ],
    targets: [
        .binaryTarget(
            name: "CLibChewing",
            url: "https://github.com/abaltatech/libchewing/releases/download/\(version)/libchewing.xcframework.zip",
            checksum: checksum
        ),
        .target(
            name: "Chewing",
            dependencies: [
                .target(name: "CLibChewing"),
            ],
            path: "src",
            resources: [
                .process("Resources"),
            ]
        ),
    ]
)
