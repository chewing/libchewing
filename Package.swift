// swift-tools-version:6.1
import PackageDescription

let package = Package(
  name: "libchewing",
  products: [
    .library(
      name: "Chewing",
      targets: ["CChewing"]
    )
  ],
  targets: [
    // Expose the existing C public headers in `capi/include` as a Clang module
    // Attach the `CargoBuild` plugin which runs `cargo build` before linking.
    .target(
      name: "CChewing",
      path: "capi",
      publicHeadersPath: "include",
      // Instruct the linker to search common Cargo target dirs where the built library may be placed
      linkerSettings: [
        .unsafeFlags(["-L", "./target/cargo-target/release"]),
        .unsafeFlags([
          "-L",
          ".build/plugins/outputs/libchewing-spm/CChewing/destination/CargoBuild/cargo-target/release",
        ]),
        .linkedLibrary("chewing_capi"),
      ],
      plugins: [
        .plugin(name: "CargoBuild")
      ]
    ),
    // Build-tool plugin that invokes cargo to produce the static library
    .plugin(
      name: "CargoBuild",
      capability: .buildTool(),
      path: "swift/tools/CargoBuildPlugin"
    ),
    // Swift test target in `swift/unit_tests` to validate C API accessibility from Swift
    .testTarget(
      name: "ChewingTests",
      dependencies: ["CChewing"],
      path: "swift/unit_tests"
    ),
  ]
)
