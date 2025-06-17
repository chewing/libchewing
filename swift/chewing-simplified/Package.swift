// swift-tools-version:5.9
import PackageDescription

let version = "1.0.1"
let checksum = "7042e93620df95cf17bf796efbcac1800bc776207f450f142d8285a6e7db1f06"

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
