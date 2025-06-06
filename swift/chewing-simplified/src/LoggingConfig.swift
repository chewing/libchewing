import Foundation

// MARK: - LogLevel

/// OptionSet representing allowed logging levels.
public struct LogLevel: OptionSet, CustomStringConvertible {
    public let rawValue: Int

    /// Log level for critical messages; use for unrecoverable errors.
    public static let critical = LogLevel(rawValue: 1 << 0)
    /// Log level for error messages indicating failures that might be handled.
    public static let error = LogLevel(rawValue: 1 << 1)
    /// Log level for warning messages indicating potential issues.
    public static let warning = LogLevel(rawValue: 1 << 2)
    /// Log level for informational messages that highlight the progress of the application.
    public static let info = LogLevel(rawValue: 1 << 3)
    /// Log level for debug messages containing detailed information for debugging.
    public static let debug = LogLevel(rawValue: 1 << 4)
    /// Log level for verbose messages containing highly detailed logging information.
    public static let verbose = LogLevel(rawValue: 1 << 5)

    /// Convenience for all known levels together.
    public static let all: LogLevel = [.critical, .error, .warning, .info, .debug, .verbose]

    /// Creates a LogLevel instance from a raw integer value.
    /// - Parameter rawValue: The bitmask value representing the log levels.
    public init(rawValue: Int) {
        self.rawValue = rawValue
    }

    /// A textual representation of the log level(s) contained in this instance.
    public var description: String {
        var parts: [String] = []
        if contains(.critical) { parts.append("critical") }
        if contains(.error)    { parts.append("error")    }
        if contains(.warning)  { parts.append("warning")  }
        if contains(.info)     { parts.append("info")     }
        if contains(.debug)    { parts.append("debug")    }
        if contains(.verbose)  { parts.append("verbose")  }
        return parts.isEmpty ? "none" : parts.joined(separator: "|")
    }
}

// MARK: - LoggingConfig

/// Configuration for ChewingWrapper’s logging subsystem.
///
/// Use this to control which levels are emitted and how they’re handled.
/// Example:
/// ```swift
/// let config = LoggingConfig(
///   enabled: true,
///   levels: [.critical, .error, .warning, .info],
///   callback: { level, message in
///     print("[\(level)] \(message)")
///   }
/// )
/// ```
public struct LoggingConfig {
    /// If `false`, no messages will be emitted, regardless of `levels`.
    public let enabled: Bool

    /// The set of levels that will actually be logged.
    /// Messages whose level is not contained here are ignored.
    public let levels: LogLevel

    /// Called whenever a message is emitted at a level contained in `levels`.
    /// If `nil`, a default fallback (e.g., `print(...)`) is used.
    public let callback: ((LogLevel, String) -> Void)?

    /// Initialises a new LoggingConfig.
    /// - Parameters:
    ///   - enabled: A Boolean flag indicating whether any logging should occur.
    ///   - levels: The set of log levels that should be emitted.
    ///             Defaults to `[.critical, .error, .warning, .info, .debug]`.
    ///   - callback: An optional closure to receive `(level, message)`
    public init(
        enabled: Bool = true,
        levels: LogLevel = [.critical, .error, .warning, .info, .debug],
        callback: ((LogLevel, String) -> Void)? = nil
    ) {
        assert(levels.rawValue & ~LogLevel.all.rawValue == 0,
               "LoggingConfig received an unknown log level bitmask")
        self.enabled = enabled
        self.levels = levels
        self.callback = callback
    }
}
