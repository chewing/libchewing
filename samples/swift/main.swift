import Foundation

// Store original terminal settings
var origTermios = termios()

@_cdecl("swift_disableRawMode")
func swift_disableRawMode() {
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &origTermios)
}

let escapeKey: Int32 = 27  // ESC

//
// Callback shims
//
@_cdecl("swift_logger_callback")
func swift_logger_callback(
    _ level: Int32,
    _ message: UnsafePointer<CChar>?
) {
    guard level != CHEWING_LOG_VERBOSE else { return }
    let msg = message.map { String(cString: $0) } ?? ""
    let lvl =
        [
            CHEWING_LOG_DEBUG: "DEBUG",
            CHEWING_LOG_INFO: "INFO",
            CHEWING_LOG_WARN: "WARN",
            CHEWING_LOG_ERROR: "ERROR",
        ][level] ?? "UNKNOWN"
    print("[chewing \(lvl)] \(msg)")
}

@_cdecl("swift_candidate_info_callback")
func swift_candidate_info_callback(
    _ pageSize: CInt,
    _ numPages: CInt,
    _ candOnPage: CInt,
    _ total: CInt,
    _ items: UnsafeMutablePointer<UnsafePointer<CChar>?>?
) {
    guard let candidates = items else { return }
    print("Candidates [\(candOnPage)/\(total)] (page \(pageSize) of \(numPages))")
    for i in 0..<Int(total) {
        if let cstrPtr = candidates[i] {
            print("  \(i): \(String(cString: cstrPtr))")
        }
    }
}

@_cdecl("swift_buffer_callback")
func swift_buffer_callback(_ buf: UnsafePointer<CChar>?) {
    if let b = buf { print("Buffer:   \(String(cString: b))") }
}

@_cdecl("swift_bopomofo_callback")
func swift_bopomofo_callback(_ buf: UnsafePointer<CChar>?) {
    if let b = buf { print("Preedit:  \(String(cString: b))") }
}

@_cdecl("swift_commit_callback")
func swift_commit_callback(_ buf: UnsafePointer<CChar>?) {
    if let b = buf { print("Commit:   \(String(cString: b))") }
}

//
// Simplified processKey: hands off to the C++ wrapper
//
func processKey(_ key: Int32) {
    cs_process_key(Int8(key))
}

//
// Application entry point
//
func main() {
    // ——— Terminal in raw mode
    tcgetattr(STDIN_FILENO, &origTermios)
    atexit(swift_disableRawMode)
    var raw = origTermios
    raw.c_lflag &= ~UInt(ECHO | ICANON)
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw)

    // ——— Prepare data path
    let dataPath = "../build/swift/data"
    let cStr = strdup(dataPath)

    // ——— Set up application context & callbacks
    var csCtx = cs_context_t()
    csCtx.config.data_path = cStr
    csCtx.config.cand_per_page = 2
    csCtx.config.max_chi_symbol_len = 18

    csCtx.callbacks.logger = swift_logger_callback
    csCtx.callbacks.candidate_info = swift_candidate_info_callback
    csCtx.callbacks.buffer = swift_buffer_callback
    csCtx.callbacks.bopomofo = swift_bopomofo_callback
    csCtx.callbacks.commit = swift_commit_callback

    guard cs_init(&csCtx) else {
        print("Failed to initialize libchewing")
        exit(1)
    }

    // ——— Main event loop
    while true {
        let c = getchar()
        if c == escapeKey { break }
        if c == "`".utf8CString[0] {
            cs_select_candidate(5)
            continue
        }
        processKey(c)
    }

    print("Program terminated.")
    cs_terminate()
}
main()
