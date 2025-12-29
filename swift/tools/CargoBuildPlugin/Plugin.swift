import Foundation
import PackagePlugin

@main
struct CargoBuildPlugin: BuildToolPlugin {
  func createBuildCommands(context: PluginContext, target: Target) throws -> [Command] {
    // Paths
    let packageDir = context.package.directoryURL
    let capiDir = packageDir.appendingPathComponent("capi")
    // Use the plugin's work directory (sandboxed) to avoid macOS permission issues
    let scratchTarget = context.pluginWorkDirectoryURL.appendingPathComponent("cargo-target")

    // Auto-build is enabled by default. To disable automatic Cargo build, set `LIBCHEWING_AUTO_BUILD_CARGO=0`.
    if ProcessInfo.processInfo.environment["LIBCHEWING_AUTO_BUILD_CARGO"] == "0" {
      return []
    }

    // Prefer common cargo install locations; if not found, abort with a clear error
    let fm = FileManager.default
    let home = fm.homeDirectoryForCurrentUser.path
    let candidates: [URL] = [
      URL(fileURLWithPath: "/usr/bin/cargo"),
      URL(fileURLWithPath: "/usr/local/bin/cargo"),
      URL(fileURLWithPath: "/opt/homebrew/bin/cargo"),
      URL(fileURLWithPath: "\(home)/.cargo/bin/cargo"),
    ]

    var cargoURL: URL? = nil
    for url in candidates {
      if fm.fileExists(atPath: url.path) {
        cargoURL = url
        break
      }
    }

    guard let cargo = cargoURL else {
      struct UserError: Error, CustomStringConvertible {
        let description: String
        init(_ s: String) { description = s }
      }
      throw UserError(
        "`cargo` not found on the system. Please install Rust (https://rustup.rs/) to enable automatic builds, or disable automatic Cargo build by setting `LIBCHEWING_AUTO_BUILD_CARGO=0` and run `swift/scripts/build-cargo.sh` manually to produce the library before running `swift build`."
      )
    }

    let manifestPath = capiDir.appendingPathComponent("Cargo.toml").path
    let targetDir = scratchTarget.path

    // Arguments: build the chewing_capi crate in release mode into the plugin workdir target
    let args = [
      "build",
      "--release",
      "--manifest-path",
      manifestPath,
      "--target-dir",
      targetDir,
    ]

    return [
      .prebuildCommand(
        displayName: "Building chewing_capi via cargo",
        executable: cargo,
        arguments: args,
        environment: [:],
        outputFilesDirectory: scratchTarget
      )
    ]
  }
}
