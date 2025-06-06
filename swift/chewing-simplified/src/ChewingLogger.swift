//
//  ChewingLogger.swift
//  Chewing
//

import Foundation
import os

import CLibChewing

/// Internal helper to route Chewing engine log callbacks through LoggingConfig.
package struct ChewingLogger {
    /// The current active logger instance. Used by the C callback to forward messages.
    private static var current: ChewingLogger?

    /// The OS logger used when no callback is provided.
    static private let osLogger = Logger(subsystem: "chewing", category: "ChewingLogger")
    
    /// The user-provided logging configuration.
    let config: LoggingConfig
    
    /// Initializes a new ChewingLogger and registers it as the active logger.
    /// - Parameter config: Configuration controlling which messages get emitted.
    init(config: LoggingConfig) {
        self.config = config
        ChewingLogger.current = self
    }
    
    /// C-compatible callback to receive raw Chewing engine log messages.
    /// - Parameters:
    ///   - level: Numeric log level from the Chewing engine.
    ///   - message: C string containing the log message.
    static let cLogger: @convention(c) (Int32, UnsafePointer<CChar>?) -> Void = { level, message in
        guard let logger = ChewingLogger.current else { return }
        // Map integer level to LogLevel
        let lvlOption: LogLevel = {
            switch level {
            case CHEWING_LOG_ERROR:   return .error
            case CHEWING_LOG_WARN:    return .warning
            case CHEWING_LOG_INFO:    return .info
            case CHEWING_LOG_DEBUG:   return .debug
            case CHEWING_LOG_VERBOSE: return .verbose
            default:                  return []
            }
        }()
        // Convert C string to Swift String
        let msgStr = message.map { String(cString: $0) } ?? ""
        // Prefix for consistency
        let logMsg = "[chewing]\(msgStr)"
        // Forward to instance method
        logger.log(level: lvlOption, message: logMsg)
    }
    
    /// Logs a message at the specified level according to the configuration.
    /// - Parameters:
    ///   - level: The mapped `LogLevel` for this message.
    ///   - message: The text to forward.
    func log(level: LogLevel, message: String) {
        guard config.enabled else { return }
        guard config.levels.contains(level) else { return }
        if let cb = config.callback {
            cb(level, message)
        } else {
            // Use OS logger as fallback
            switch level {
            case .critical:
                ChewingLogger.osLogger.critical("\(message)")
            case .error:
                ChewingLogger.osLogger.error("\(message)")
            case .warning:
                ChewingLogger.osLogger.warning("\(message)")
            case .info:
                ChewingLogger.osLogger.info("\(message)")
            case .debug, .verbose:
                ChewingLogger.osLogger.debug("\(message)")
            default:
                ChewingLogger.osLogger.notice("Unknown log level: \(level), \(message)")
            }
        }
    }
}
