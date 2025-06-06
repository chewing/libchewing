// swift-tools-version:5.9
import PackageDescription

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
            path: "libchewing.xcframework"
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
