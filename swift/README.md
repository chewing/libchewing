# libchewing Swift Integration

This directory contains everything needed to integrate and use the [libchewing](https://github.com/chewing/libchewing) library within Swift-based projects. It is organized into subfolders and scripts to streamline building, packaging, and demonstrating libchewing on Apple platforms (iOS, macOS, etc.).

---

## Table of Contents

- [libchewing Swift Integration](#libchewing-swift-integration)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Prerequisites](#prerequisites)
  - [Building libchewing](#building-libchewing)
    - [Using CMake](#using-cmake)
  - [Swift Package](#swift-package)
    - [1. Add as a Local Package Dependency](#1-add-as-a-local-package-dependency)
    - [2. Add as a Remote Git Dependency](#2-add-as-a-remote-git-dependency)
      - [Package Targets](#package-targets)
  - [Sample App](#sample-app)
    - [Location and Purpose](#location-and-purpose)
    - [Running the Sample App](#running-the-sample-app)
  - [Resources](#resources)
  - [Troubleshooting](#troubleshooting)
  - [Licensing](#licensing)

---

## Overview

`libchewing` is an open-source library for Zhuyin input support. This folder provides:

- **Build Scripts & CMake Files**: Utilities to build the `XCFramework` needed for iOS and Simulators.
- **Swift Wrapper**: A Swift-based wrapper (`src/`) exposing a Swifty API (`ChewingWrapper.swift`, `ChewingKey.swift`, `LoggingConfig.swift`) to the underlying C library.
- **Swift Package**: A `Package.swift` manifest to import `libchewing` into your own project.
- **Sample App**: A minimal Swift sample application (under `Examples/ChewingSampleApp`) demonstrating how to initialize and use the wrapper in a real iOS app.

---

## Prerequisites

Before building or using this integration, ensure you have:

- **Xcode 13.0+** (macOS 11.0+ SDK) or a compatible version of Xcode for your target platform.
- **Swift 5.5+** (bundled with Xcode 13+) for Swift Package Manager support.
- **Rust** to compile the `libchewing` library.
- **CMake 3.18+** to compile the `libchewing` library and package it as an `XCFramework`.
- **ios-cmake** a local verion of [`ios-cmake`](https://github.com/leetal/ios-cmake) is required to compile the project for iOS
- **Homebrew** (recommended) to install CMake, Rust, or other dependencies easily:

---

## Building libchewing

### Using CMake

To build the `XCFramework` from the original C/C++/Rust sources follow these steps:

1. **Install dependencies** (e.g., CMake, Rust).
2. **Run the build script**:
   ```bash
   ./build-xcframework.sh
   ```
    Before running the script make sure to update the path for `ios-cmake` in the script.

## Swift Package

The `chewing-simplified` folder is already a valid Swift Package. You can consume it in two ways:

### 1. Add as a Local Package Dependency

In your own `Package.swift`, add:

```swift
.package(
  name: "Chewing",
  path: "/path/to/swift/chewing-simplified"
),
```

Then, add `"ChewingWrapper"` under the dependencies for your target:

```swift
.target(
  name: "MyApp",
  dependencies: [
    .product(name: "Chewing", package: "Chewing")
  ]
)
```

### 2. Add as a Remote Git Dependency

TODO

#### Package Targets

- **`Chewing`** (Library target).
  Contains all Swift wrapper code (`ChewingWrapper.swift`, `ChewingKey.swift`, `LoggingConfig.swift`).
  **Requires** `libchewing.xcframework` to be compiled.

---

## Sample App

### Location and Purpose

The `Examples/ChewingSampleApp` folder contains a minimal SwiftUI application that demonstrates:

- How to configure the `LoggingConfig`.
- How to initialize `ChewingWrapper`.
- How to handle callbacks (`onCandidateUpdate`, `onCommit`, `onBufferUpdate`, `onPreeditUpdate`).

This sample app is a good reference if you want to integrate `libchewing` into an iOS or macOS application.

### Running the Sample App

1. Open `Examples/ChewingSampleApp/ChewingSampleApp.xcodeproj` in Xcode.
2. Update the Team under Signing & Capabilities -> Signing for the `ChewingSampleApp` target
3. Select a run destination (e.g., “iPhone 13 Pro Max Simulator”).
4. Build and run (`⌘R`).
   - The app will load the runtime data files from `Resources/data`.
   - You can type into the demo input field to see how the Chewing engine converts keystrokes into Chinese characters.

---

## Resources

- **Runtime Data Files** (`src/Resources/data/`):
  Contains binary dictionary and symbol files required by `libchewing`. Ensure these are packaged into your app’s bundle under the same relative path (or update file paths accordingly in your code).

- **C Headers** (`libchewing.xcframework/.../Headers/`):
  - `chewing.h` – primary header for the libchewing API.
  - `chewing-simplified.h` – helper functions that simplify common tasks.
  - `module.modulemap` – exposes the C APIs to Swift.

---

## Troubleshooting

- **Missing `libchewing.dylib` at runtime**:
  - Make sure `Chewing` is added to your Xcode project under “Frameworks, Libraries, and Embedded Content.”
  - Set “Embed & Sign” for the `Chewing` if you’re targeting an app bundle.

- **SwiftPM Cannot Find `libchewing.xcframework`**:
  - Ensure that `libchewing.xcframework` is located at the same folder level as `Package.swift`.
  - Alternatively, update `Package.swift` to reference a custom `binaryTarget` for a remote `XCFramework`.

- **Build Failures Related to Header Files**:
  - Verify that your search paths include the correct header directory inside `libchewing.xcframework/*/Headers`.
  - Check that the modulemap file in `Headers/` is correct and that SwiftPM can locate it.

- **Data Files Not Found at Runtime**:
  - Double-check that your app bundle includes the `Resources/data` folder.
  - Use `Bundle.main.url(forResource: "mini", withExtension: "dat", subdirectory: "data")` (or equivalent) to load runtime data.

---

## Licensing

- The `libchewing` library itself is released under the **GNU Lesser General Public License v2.1** or later. Refer to the [license](../COPYING) in the root folder of the project.
- The Swift wrapper code in this repository is provided under the same terms.