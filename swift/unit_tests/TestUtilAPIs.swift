import Foundation
import Testing

@testable import CChewing

#if os(Linux)
  import Glibc
#else
  import Darwin
#endif

// Cross-platform env setter
func setEnv(_ key: String, _ value: String) {
  #if os(Linux)
    Glibc.setenv(key, value, 1)
  #else
    Darwin.setenv(key, value, 1)
  #endif
}

@MainActor
class TestBaseClass {
  @MainActor
  // Helper: minimal keystroke parser for test strings like "<D>", "<L>", "<EE>", etc.
  func type_keystroke_by_string(_ ctx: OpaquePointer?, _ keystroke: String) {
    var i = keystroke.startIndex
    while i < keystroke.endIndex {
      if keystroke[i] == "<" {
        // parse token until '>'
        let j = keystroke[i...].firstIndex(of: ">") ?? keystroke.index(after: i)
        let token = String(keystroke[keystroke.index(after: i)..<j])
        switch token {
        case "L": chewing_handle_Left(ctx)
        case "R": chewing_handle_Right(ctx)
        case "U": chewing_handle_Up(ctx)
        case "D": chewing_handle_Down(ctx)
        case "E": chewing_handle_Enter(ctx)
        case "EE": chewing_handle_Esc(ctx)
        case "H": chewing_handle_Home(ctx)
        case "B": chewing_handle_Backspace(ctx)
        case "DC": chewing_handle_Del(ctx)
        case "SS": chewing_handle_ShiftSpace(ctx)
        case "TT": chewing_handle_DblTab(ctx)
        case "SL": chewing_handle_ShiftLeft(ctx)
        case "SR": chewing_handle_ShiftRight(ctx)
        case "PU": chewing_handle_PageUp(ctx)
        case "PD": chewing_handle_PageDown(ctx)
        case "CB": chewing_handle_Capslock(ctx)
        case "T": chewing_handle_Tab(ctx)
        default:
          // Unknown token, ignore
          break
        }
        i = keystroke.index(after: j)
      } else {
        let ch = keystroke[i]
        if let ascii = ch.asciiValue {
          chewing_handle_Default(ctx, Int32(ascii))
        } else {
          // For non-ascii, iterate UTF8 bytes
          let scalarString = String(ch).utf8
          for b in scalarString { chewing_handle_Default(ctx, Int32(b)) }
        }
        i = keystroke.index(after: i)
      }
    }
  }
  @MainActor
  func ensureTestEnv() {
    // Set test environment variables similar to C test main
    let cwd = FileManager.default.currentDirectoryPath
    let srcData = "\(cwd)/tests/data"
    if ProcessInfo.processInfo.environment["CHEWING_PATH"] == nil {
      setEnv("CHEWING_PATH", srcData)
    }
    if ProcessInfo.processInfo.environment["CHEWING_USER_PATH"] == nil {
      setEnv("CHEWING_USER_PATH", "\(cwd)/.scratch/tests")
    }

    // If a gold sqlite DB exists, create a small scratch dir that contains the
    // sqlite DB and supporting files so loader prefers the sqlite DB instead
    // of small/empty .dat tries present in tests/data.
    let golden = URL(fileURLWithPath: srcData).appendingPathComponent("golden-chewing.sqlite3")
    if FileManager.default.fileExists(atPath: golden.path) {
      let scratch = URL(fileURLWithPath: cwd).appendingPathComponent("scratch")
      // Clean and create scratch
      try? FileManager.default.removeItem(at: scratch)
      do {
        try FileManager.default.createDirectory(at: scratch, withIntermediateDirectories: true)
        // Copy only the golden sqlite and essential support files
        let candidates = [
          "golden-chewing.sqlite3", "symbols.dat", "swkb.dat", "phone.cin", "pinyin.tab",
        ]
        for f in candidates {
          let s = URL(fileURLWithPath: srcData).appendingPathComponent(f).path
          let d = scratch.appendingPathComponent(f).path
          if FileManager.default.fileExists(atPath: s) {
            try? FileManager.default.copyItem(atPath: s, toPath: d)
          }
        }
        setEnv("CHEWING_PATH", scratch.path)
        print("CHEWING_PATH set to \(scratch.path) (using golden sqlite)")
      } catch {
        print("Failed to create scratch directory: \(error)")
        // fallback: leave CHEWING_PATH as tests/data
      }
    }

    // DEBUG: print env values
    print("CHEWING_PATH=\(ProcessInfo.processInfo.environment["CHEWING_PATH"] ?? "(unset)")")
    print(
      "CHEWING_USER_PATH=\(ProcessInfo.processInfo.environment["CHEWING_USER_PATH"] ?? "(unset)")")
    // Enable rust tracing logs for debugging dictionary loading if unspecified
    if ProcessInfo.processInfo.environment["RUST_LOG"] == nil {
      setEnv("RUST_LOG", "chewing=trace")
      print("RUST_LOG set to chewing=trace")
    }
  }
  @MainActor
  func clean_userphrase() {
    ensureTestEnv()
    // Remove the userphrase DB file if present: CHEWING_USER_PATH/chewing.dat
    if let userPath = ProcessInfo.processInfo.environment["CHEWING_USER_PATH"] {
      let db = URL(fileURLWithPath: userPath).appendingPathComponent("chewing.dat").path
      try? FileManager.default.removeItem(atPath: db)
    }
  }
  @MainActor
  func ueStrLen(_ cstr: UnsafePointer<CChar>?) -> Int {
    guard let cstr else { return 0 }
    return String(cString: cstr).count
  }

  @MainActor
  func ok_preedit_buffer(_ ctx: OpaquePointer?, _ expected: String) {
    if let b = chewing_buffer_String(ctx) {
      let s = String(cString: b)
      #expect(s == expected, "preedit buffer should be '\(expected)' but was '\(s)'")
      chewing_free(UnsafeMutableRawPointer(mutating: b))
    } else {
      #expect(Bool(false), "chewing_buffer_String returned NULL")
    }
  }
  @MainActor
  func ok_bopomofo_buffer(_ ctx: OpaquePointer?, _ expected: String) {
    if let b = chewing_bopomofo_String(ctx) {
      let s = String(cString: b)
      #expect(s == expected, "bopomofo buffer should be '\(expected)' but was '\(s)'")
      chewing_free(UnsafeMutableRawPointer(mutating: b))
    } else {
      #expect(Bool(false), "chewing_bopomofo_String returned NULL")
    }
  }
}

@MainActor
class DataBackedTestSuite: TestBaseClass {

  override init() {
    // Minimal environment setup for C API accessibility tests.
    // Keep it lightweight and cross-platform to run under Linux swift images.
    let cwd = FileManager.default.currentDirectoryPath
    let scratch = URL(fileURLWithPath: cwd).appendingPathComponent("scratch")

    // Ensure a clean scratch directory
    try? FileManager.default.removeItem(at: scratch)
    do {
      try FileManager.default.createDirectory(at: scratch, withIntermediateDirectories: true)
    } catch {
      print("Failed to create scratch directory: \(error)")
    }

    // Prefer repository data dir if present, otherwise use tests/data
    let repoDataDir = "\(cwd)/data/dict/chewing"
    let srcDir =
      FileManager.default.fileExists(atPath: repoDataDir) ? repoDataDir : "\(cwd)/tests/data"

    setEnv("CHEWING_PATH", srcDir)

    // create a userpath under scratch
    let userScratch = scratch.appendingPathComponent("tests")
    try? FileManager.default.createDirectory(at: userScratch, withIntermediateDirectories: true)
    setEnv("CHEWING_USER_PATH", userScratch.path)

    print("CHEWING_PATH set to \(srcDir), CHEWING_USER_PATH=\(userScratch.path)")
  }

  deinit {
    // Remove the temporary scratch database after the test suite finishes
    let cwd = FileManager.default.currentDirectoryPath
    let scratch = URL(fileURLWithPath: cwd).appendingPathComponent("scratch")
    try? FileManager.default.removeItem(at: scratch)
  }
}
