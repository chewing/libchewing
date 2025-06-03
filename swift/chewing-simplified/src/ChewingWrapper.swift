@_implementationOnly import CLibChewing
import Foundation

// MARK: - ChewingWrapperError

/// Errors that can occur when initializing or using the ChewingWrapper.
public enum ChewingWrapperError: Error {
    case notFound
    case initializationFailed
}

// MARK: - ChewingWrapper

/// A Swift wrapper around the native Chewing C API (CLibChewing).
///
/// This class manages the lifecycle of the Chewing input context,
/// forwards keystrokes to the library, and dispatches callback events
/// (buffer updates, candidate lists, commits, and logging) to Swift closures.
public class ChewingWrapper {
    // These are Swift closures that user code can set:
    public var onCandidateUpdate: (([String]) -> Void)?
    public var onCommit: ((String) -> Void)?
    public var onBufferUpdate: ((String) -> Void)?
    public var onPreeditUpdate: ((String) -> Void)?
    public var loggingCallback: ((Int, String) -> Void)?
    public var loggingEnabled: Bool = true

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
    /// - Throws: `ChewingWrapperError.notFound` if the data directory cannot be located.
    ///           `ChewingWrapperError.initializationFailed` if the native `cs_init` call fails.
    public init(candPerPage: Int = 10, maxChiSymbolLen: Int = 18, dataDirectoryPath: String? = nil) throws {
        let dataDirectoryPath = dataDirectoryPath ?? ChewingWrapper.dataDirectoryPath
        guard let dataDirectoryPath else {
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
            logger: ChewingWrapper.loggerHandler
        )

        ctx = cs_context_s(config: config, callbacks: callbacks)

        // Register this instance for callback routing
        ChewingWrapper.currentWrapper = self

        // Call cs_init
        isInitialized = cs_init(&ctx)
        if !isInitialized {
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

        // convert Character to char
        let scalarVal = String(key).utf8CString
        guard let cChar = scalarVal.first else { return }
        cs_process_key(cChar)
    }

    /// Selects a candidate word by its index in the current candidate list.
    ///
    /// - Parameter index: Zero-based index of the candidate to commit.
    ///                    The Chewing engine will commit that candidate to the buffer.
    public func selectCandidate(at index: Int) {
        guard isInitialized else { return }

        cs_select_candidate(Int32(index))
    }

    private var ctx: cs_context_s
    private var isInitialized: Bool = false
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

private extension ChewingWrapper {
    /// Holds a reference to the most recently initialized wrapper, used by C callbacks.
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

    /// C callback invoked for logging messages from the Chewing engine.
    ///
    /// - Parameters:
    ///   - level: Numeric log level (e.g., `CHEWING_LOG_DEBUG`, `CHEWING_LOG_INFO`).
    ///   - message: C string containing the log message.
    private static let loggerHandler: @convention(c) (Int32, UnsafePointer<CChar>?) -> Void = { level, message in
        guard let wrapper = ChewingWrapper.currentWrapper else { return }
        guard level != CHEWING_LOG_VERBOSE, wrapper.loggingEnabled else { return }
        let msg = message.map { String(cString: $0) } ?? ""
        let lvl =
        [
            CHEWING_LOG_DEBUG: "DEBUG",
            CHEWING_LOG_INFO: "INFO",
            CHEWING_LOG_WARN: "WARN",
            CHEWING_LOG_ERROR: "ERROR",
        ][level] ?? "UNKNOWN"
        let logMSG = "[chewing \(lvl)] \(msg)"
        wrapper.loggingCallback?(Int(level), logMSG)
        print(logMSG)
    }
}
