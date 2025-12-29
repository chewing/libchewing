import Foundation
import Testing

@testable import CChewing

@MainActor
struct ChewingTestsSuite1 {
  @Test func testCreateDelete() {
    let ctx = chewing_new()
    #expect(ctx != nil, "chewing_new() should return non-null context")
    if ctx != nil {
      chewing_delete(ctx)
    }
  }

  @Test func testDefaultDictionaryNames() {
    let names = chewing_get_defaultDictionaryNames()
    #expect(names != nil, "chewing_get_defaultDictionaryNames() should return non-null")
    if let names = names {
      let s = String(cString: names)
      #expect(!s.isEmpty, "Default dictionary names should not be empty")
    }
  }

  @Test func testSetGetKBType() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(ctx != nil, "chewing_new() should return non-null context")
      return
    }
    defer { chewing_delete(ctx) }

    // KB_HSU == 1 according to C header enum
    let rc = chewing_set_KBType(ctx, 1)
    #expect(rc == 0, "chewing_set_KBType should return 0 on success")

    let kb = chewing_get_KBType(ctx)
    #expect(kb == 1, "chewing_get_KBType should reflect set value")
  }
}
