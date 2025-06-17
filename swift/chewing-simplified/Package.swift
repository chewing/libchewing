// swift-tools-version:5.9
import PackageDescription

let version = "1.0.2"
let checksum = "4aa3c9fe0edec4f0cc3c503ec90d1e9a664cd375cf91e744d244e387b028383c"

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
