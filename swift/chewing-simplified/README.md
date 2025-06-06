

# Chewing Swift Package

A Swift Package that wraps the [libchewing](https://github.com/chewing/libchewing) input engine for Taiwanese Mandarin, providing a Swifty API and bundling a prebuilt XCFramework for iOS. Use this package to integrate Chewing into your Swift-based projects via Swift Package Manager.

---

## Table of Contents

- [Chewing Swift Package](#chewing-swift-package)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Requirements](#requirements)
  - [Package Structure](#package-structure)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Documentation](#documentation)
    - [Viewing the Documentation](#viewing-the-documentation)
  - [Resources \& Runtime Data](#resources--runtime-data)
  - [Troubleshooting](#troubleshooting)
  - [License](#license)

---

## Overview

The **Chewing** Swift Package provides:

- A **binary target** (`CLibChewing`) that embeds `libchewing.xcframework` (prebuilt for iOS device and simulator).
- A **Swift wrapper target** (`Chewing`) under `src/` that:
  - Exposes a `ChewingWrapper` class to manage the native Chewing context.
  - Provides key constants (`ChewingKey.swift`) for feeding input events.
  - Allows configurable logging via `LoggingConfig.swift`.
  - Bridges C callbacks (buffer updates, candidate lists, commits, logging) into Swift.

By depending on this package, you can integrate the Chewing input engine into your iOS apps, process keystrokes, observe candidate updates, and commit Chinese text.

---

## Requirements

- **Xcode 14.0+** (Swift 5.9+ toolchain)
- **iOS 16.0+** as specified in `Package.swift`
- **Swift Package Manager** (bundled with Xcode)

> The package is currently configured for iOS only. If you wish to add macOS support, see the Note in the Troubleshooting section.

---

## Package Structure

```
chewing-simplified/
├── Package.swift                 # Package manifest
├── README.md                     # (You are here)
├── libchewing.xcframework/       # Prebuilt XCFramework (iOS device & simulator)
└── src/
    ├── ChewingWrapper.swift      # Main Swift wrapper interface
    ├── ChewingKey.swift          # Key constants (e.g. .enter, .space, .backspace)
    ├── LoggingConfig.swift       # Logging configuration (LogLevel, LoggingConfig)
    └── Resources/
        └── data/                 # Binary data files (mini.dat, tsi.dat, word.dat, etc.)
```

- **`Package.swift`**
  - Defines two targets:
    - `CLibChewing` (binary target pointing to `libchewing.xcframework`)
    - `Chewing` (Swift target under `src/`, depends on `CLibChewing`, includes resources)
- **`libchewing.xcframework`**
  - Contains the compiled Chewing engine for both device (arm64) and simulator (arm64_x86_64).
- **`src/`**
  - Swift sources for `ChewingWrapper`, `ChewingKey`, and `LoggingConfig`.
  - Runtime data under `Resources/data/` which is automatically bundled by SwiftPM.

---

## Installation

To use Chewing in your Swift Package Manager project:

1. **Add as a dependency** in your target `Package.swift`:

   ```swift
   // In your Package.swift
   .package(
     url: "https://github.com/<TBD>.git",
     from: "1.0.0"
   ),
   ```

2. **Add `Chewing` to your target dependencies**:

   ```swift
   .target(
     name: "MyApp",
     dependencies: [
       .product(name: "Chewing", package: "chewing-simplified")
     ]
   ),
   ```

3. **Run**:

   ```bash
   swift package update
   swift build
   ```

---

## Usage

After installing, import the module and initialize the wrapper:

```swift
import Chewing

// 1. Configure logging (optional)
let loggingConfig = LoggingConfig(
    enabled: true,
    levels: [.error, .warning, .info, .debug],
    callback: { level, msg in
        print("[Chewing][\(level)]: \(msg)")
    }
)

do {
    // 2. Initialize ChewingWrapper
    //    Passing `nil` for dataDirectoryPath uses `Bundle.module.resourcePath` automatically.
    let chewing = try ChewingWrapper(
        candPerPage: 10,
        maxChiSymbolLen: 18,
        dataDirectoryPath: nil,
        loggingConfig: loggingConfig
    )

    // 3. Observe buffer/preedit updates
    chewing.onBufferUpdate = { bufferText in
        // update UI with bufferText
    }
    chewing.onPreeditUpdate = { preeditText in
        // show current composition text
    }
    // 4. Observe candidate list
    chewing.onCandidateUpdate = { candidates in
        // update candidate UI
    }
    // 5. Observe committed text
    chewing.onCommit = { committed in
        // append committed text to final string
    }

    // 6. Process keystrokes (e.g. in a UITextView delegate)
    chewing.process(key: "a")        // ASCII letter
    chewing.process(key: ChewingKey.space)   // Space key
    chewing.process(key: ChewingKey.enter)   // Enter key
} catch {
    print("Failed to initialize Chewing: \(error)")
}
```

To select a candidate by index (0-based):
```swift
chewing.selectCandidate(at: 1)
```

---

## Documentation

This package ships with a DocC archive (`Chewing.doccarchive`) that contains API reference generated from the `///` comments in `ChewingWrapper.swift`, `ChewingKey.swift`, and `LoggingConfig.swift`.

### Viewing the Documentation

1. **Open the DocC archive** in Finder:
   - Locate `Chewing.doccarchive` in your local clone (it may be in a directory you exported it to).
   - Double-click the `.doccarchive` to open it in Xcode’s documentation viewer.
2. **Browse the API Reference** within Xcode:
   - Use the sidebar to navigate between modules, symbols, and detailed descriptions.

---

## Resources & Runtime Data

The `Resources/data/` folder contains the five `.dat` files required by the Chewing engine:

- `mini.dat`
- `swkb.dat`
- `symbols.dat`
- `tsi.dat`
- `word.dat`

SwiftPM automatically bundles these files under `Bundle.module/resourcePath/data/`. At runtime, when you pass `nil` for `dataDirectoryPath`, `ChewingWrapper` will load data from `Bundle.module.resourcePath + "/data"`.

---

## Troubleshooting

- **“no such module 'CLibChewing'”** when building in Xcode:
  - Ensure your scheme’s destination is set to an **iOS** simulator or device, since the XCFramework only includes iOS slices.
- **Data files not found**:
  - Confirm the `data` folder is included in your app bundle. Use:
    ```swift
    let url = Bundle.module.url(forResource: "mini", withExtension: "dat", subdirectory: "data")
    ```
- **DocC archive not found**:
  - If you do not see `Chewing.doccarchive`, run **Product → Build Documentation** in Xcode for the “Chewing” scheme.

> **Note:** This package currently supports iOS only. To add macOS support, rebuild `libchewing.xcframework` with a macOS slice and update `Package.swift` to include `.macOS(.v12)` in the `platforms` array.

---

## License

- The `libchewing` engine inside `libchewing.xcframework` is licensed under the **GNU Lesser General Public License v2.1** (or later). See its [LICENSE](https://github.com/chewing/libchewing/blob/master/LICENSE).
- The Swift wrapper code (`src/`) and this README are provided under the same license.