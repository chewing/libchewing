//
//  ChewingWrapper.swift
//  Chewing
//

import CLibChewing
import Foundation

// MARK: - ChewingWrapperError

/// Errors that can occur when initializing or using the ChewingWrapper.
public enum ChewingWrapperError: Error {
    /// Indicates that the Chewing data directory could not be located.
    case notFound
    /// Indicates that initialization of the native Chewing engine failed.
    case initializationFailed
}

// MARK: - ChewingWrapper

/// A Swift wrapper around the native Chewing C API (CLibChewing).
///
/// This class manages the lifecycle of the Chewing input context,
/// forwards keystrokes to the library, and dispatches callback events
/// (buffer updates, candidate lists, commits, and logging) to Swift closures.
public class ChewingWrapper {
    /// These are Swift closures that user code can set:
    /// Closure invoked when the Chewing engine generates a new list of candidates.
    /// - Parameter candidates: An array of candidate strings from the engine.
    public var onCandidateUpdate: (([String]) -> Void)?
    /// Closure invoked when the Chewing engine commits text to the application.
    /// - Parameter committedText: The string that was committed.
    public var onCommit: ((String) -> Void)?
    /// Closure invoked when the Chewing engine’s composed buffer is updated.
    /// - Parameter bufferText: The current content of the composition buffer.
    public var onBufferUpdate: ((String) -> Void)?
    /// Closure invoked when the Chewing engine’s preedit (in-progress composition) text changes.
    /// - Parameter preeditText: The current preedit text.
    public var onPreeditUpdate: ((String) -> Void)?

    /// Returns the file system path to the directory containing Chewing data files.
    ///
    /// The Chewing package should include required `.dat` resources under
    /// its SwiftPM `resources:` configuration. This path is used when
    /// initializing the Chewing context.
    public static var dataDirectoryPath: String? {
        return Bundle.module.resourcePath
    }

    /// Initializes a new ChewingWrapper instance.
    ///
    /// - Parameters:
    ///   - candPerPage: Number of candidate words to display per page (default is 10).
    ///   - maxChiSymbolLen: Maximum length of a Chinese symbol sequence (default is 18).
    ///   - dataDirectoryPath: Optional override for the Chewing data directory path.
    ///                        If `nil`, `ChewingWrapper.dataDirectoryPath` is used.
    ///   - loggingConfig: Configuration for logging behaviour.
    ///                    If logging is enabled but no callback is provided an internal logger will be used
    /// - Throws: `ChewingWrapperError.notFound` if the data directory cannot be located.
    ///           `ChewingWrapperError.initializationFailed` if the native `cs_init` call fails.
    public required init(candPerPage: Int = 10,
                         maxChiSymbolLen: Int = 18,
                         dataDirectoryPath: String? = nil,
                         loggingConfig: LoggingConfig) throws
    {
        // Initialize the internal logger to route Chewing logs
        logger = ChewingLogger(config: loggingConfig)

        guard let dataDirectoryPath = dataDirectoryPath ?? ChewingWrapper.dataDirectoryPath else {
            logger.log(level: .critical, message: "Failed to retrieve data directory path")
            throw ChewingWrapperError.notFound
        }

        let config = cs_config_t(
            data_path: strdup(dataDirectoryPath),
            cand_per_page: Int32(candPerPage),
            max_chi_symbol_len: Int32(maxChiSymbolLen)
        )

        let callbacks = cs_callbacks_s(
            candidate_info: ChewingWrapper.candidateInfoHandler,
            buffer: ChewingWrapper.bufferHandler,
            bopomofo: ChewingWrapper.preeditHandler,
            commit: ChewingWrapper.commitHandler,
            logger: ChewingLogger.cLogger
        )

        ctx = cs_context_s(config: config, callbacks: callbacks)

        // Register this instance for callback routing
        ChewingWrapper.currentWrapper = self

        // Call cs_init
        isInitialized = cs_init(&ctx)
        if !isInitialized {
            logger.log(level: .critical, message: "Failed to initialize Chewing")
            throw ChewingWrapperError.initializationFailed
        }
    }

    deinit {
        if isInitialized {
            _ = cs_terminate()
            ctx = cs_context_s()
        }
    }

    /// Sends a single key input to the Chewing engine.
    ///
    /// - Parameter key: A `Character` representing a keystroke (e.g., a letter, space, backspace, or enter).
    ///                  This is converted to a C `char` and forwarded to `cs_process_key`.
    public func process(key: Character) {
        guard isInitialized else { return }
        guard let cKey = key.asciiValue else { return }

        cs_process_key(CChar(cKey))
    }

    /// Selects a candidate word by its index in the current candidate list.
    ///
    /// - Parameter index: Zero-based index of the candidate to commit.
    ///                    The Chewing engine will commit that candidate to the buffer.
    public func selectCandidate(at index: Int) {
        guard isInitialized else { return }

        cs_select_candidate(Int32(index))
    }

    private var ctx: cs_context_s = .init()
    private var isInitialized: Bool = false
    private var logger: ChewingLogger
}

public extension ChewingWrapper {
    /// Sends a key input to the Chewing engine using a `ChewingKey` enum value.
    ///
    /// - Parameter key: A `ChewingKey` value (e.g., `.enter`, `.space`, `.backspace`).
    ///                  The underlying `CChar` is extracted and sent to `cs_process_key`.
    func process(key: ChewingKey) {
        guard isInitialized else { return }

        cs_process_key(key.cValue)
    }
}

// MARK: Private extensions

private extension ChewingWrapper {
    /// Holds a weak reference to the most recently initialized ChewingWrapper instance,
    /// used for routing callback invocations from the C library into the Swift closures.
    private weak static var currentWrapper: ChewingWrapper?

    // MARK: - C callback entry points (bridge to instance closures)

    /// C callback invoked when the Chewing engine has generated a list of candidates.
    ///
    /// - Parameters:
    ///   - pageSize: Number of candidates per page (unused).
    ///   - numPages: Total number of pages (unused).
    ///   - candOnPage: Index of the current page (unused).
    ///   - total: Total number of candidates available.
    ///   - items: C array of C strings representing candidate words.
    private static let candidateInfoHandler: @convention(c) (Int32, Int32, Int32, Int32, UnsafeMutablePointer<UnsafePointer<CChar>?>?) -> Void = { _, _, _, total, items in
        guard let wrapper = ChewingWrapper.currentWrapper else { return }
        var candidates: [String] = []
        if let items = items {
            for i in 0 ..< Int(total) {
                if let cStrPtr = items[i] {
                    candidates.append(String(cString: cStrPtr))
                }
            }
        }
        wrapper.onCandidateUpdate?(candidates)
    }

    /// C callback invoked when the Chewing engine’s buffer (composed text) is updated.
    ///
    /// - Parameter buf: C string containing the current buffer content.
    private static let bufferHandler: @convention(c) (UnsafePointer<CChar>?) -> Void = { buf in
        guard let wrapper = ChewingWrapper.currentWrapper, let buf else { return }
        let str = String(cString: buf)
        wrapper.onBufferUpdate?(str)
    }

    /// C callback invoked when the Chewing engine’s preedit (in-progress composition) is updated.
    ///
    /// - Parameter buf: C string containing the current preedit text.
    private static let preeditHandler: @convention(c) (UnsafePointer<CChar>?) -> Void = { buf in
        guard let wrapper = ChewingWrapper.currentWrapper, let buf else { return }
        let str = String(cString: buf)
        wrapper.onPreeditUpdate?(str)
    }

    /// C callback invoked when the Chewing engine commits text to the application.
    ///
    /// - Parameter buf: C string containing the committed text.
    private static let commitHandler: @convention(c) (UnsafePointer<CChar>?) -> Void = { buf in
        guard let wrapper = ChewingWrapper.currentWrapper, let buf else { return }
        let str = String(cString: buf)
        wrapper.onCommit?(str)
    }
}
