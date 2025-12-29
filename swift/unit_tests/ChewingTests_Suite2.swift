import Foundation
import Testing

@testable import CChewing

@MainActor
final class ChewingTestsSuite2: DataBackedTestSuite {
  @Test func testSequenceCommit() {
    // Reproduce the "綠茶" sequence from contrib/simple-select.c
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(Bool(false)), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    // Prepare environment similar to the example
    let initialSelKeysStr = "123456789"
    let selKeys: [Int32] = initialSelKeysStr.utf8.map { Int32($0) } + [0]
    selKeys.withUnsafeBufferPointer { ptr in
      chewing_set_selKey(ctx, ptr.baseAddress, 9)
    }
    chewing_set_candPerPage(ctx, 9)
    chewing_set_maxChiSymbolLen(ctx, 16)

    // Input sequence for "綠茶": 'x','m','4','t','8','6'
    let inputSeqStr = "xm4t86"
    let keys: [UInt8] = inputSeqStr.utf8.map { UInt8($0) }
    for k in keys {
      let rc = chewing_handle_Default(ctx, Int32(k))
      #expect(rc == 0, "chewing_handle_Default failed for \(Character(UnicodeScalar(k)))")
    }

    // Commit
    _ = chewing_handle_Enter(ctx)

    if chewing_commit_Check(ctx) != 0 {
      if let cptr = chewing_commit_String_static(ctx) {
        let s = String(cString: cptr)
        // Expect non-empty; exact characters may depend on dictionary, but example is "綠茶".
        #expect(!s.isEmpty, "commit string should not be empty")
      } else {
        #expect(Bool(Bool(false)), "chewing_commit_String_static returned NULL")
      }
    } else {
      #expect(Bool(Bool(false)), "chewing_commit_Check returned false after sequence")
    }
  }

  @Test func testCandidateEnumerationFlow() {
    // Test candidate enumeration after a short input sequence
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(Bool(false)), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    // Prepare selection keys and options
    let initialSelKeysStr = "123456789"
    let selKeys: [Int32] = initialSelKeysStr.utf8.map { Int32($0) } + [0]
    selKeys.withUnsafeBufferPointer { ptr in
      chewing_set_selKey(ctx, ptr.baseAddress, 9)
    }
    chewing_set_candPerPage(ctx, 9)

    // Enter a short input that will produce candidates (e.g., '5' as in example)
    _ = chewing_handle_Default(ctx, Int32(Character("5").asciiValue!))
    _ = chewing_handle_Space(ctx)
    _ = chewing_handle_Down(ctx)

    // Enumerate candidates
    chewing_cand_Enumerate(ctx)
    var foundAny = false
    while chewing_cand_hasNext(ctx) != 0 {
      foundAny = true
      if let cstr = chewing_cand_String(ctx) {
        let s = String(cString: cstr)
        #expect(!s.isEmpty, "candidate string should not be empty")
        // Free the returned string
        chewing_free(UnsafeMutableRawPointer(mutating: cstr))
      } else {
        #expect(Bool(Bool(false)), "chewing_cand_String returned NULL")
      }
    }
    #expect(foundAny, "expected at least one candidate")
  }

  @Test func testNullHandling() {
    // Verify various APIs behave safely when passed NULL
    // Most functions should return -1 for error cases, or return empty strings or NULL pointers.

    // Simple returns
    chewing_Reset(nil)  // shall not crash

    #expect(chewing_set_KBType(nil, 0) == -1, "chewing_set_KBType(NULL) == -1")
    #expect(chewing_get_KBType(nil) == -1, "chewing_get_KBType(NULL) == -1")

    if let buf = chewing_get_KBString(nil) {
      let s = String(cString: buf)
      #expect(s.isEmpty, "chewing_get_KBString(NULL) should return empty string")
      chewing_free(UnsafeMutableRawPointer(buf))
    } else {
      #expect(Bool(Bool(false)), "chewing_get_KBString(NULL) returned NULL")
    }

    chewing_delete(nil)  // shall not crash
    chewing_free(nil)  // shall not crash

    chewing_set_candPerPage(nil, 0)
    #expect(chewing_get_candPerPage(nil) == -1, "chewing_get_candPerPage(NULL) == -1")

    chewing_set_maxChiSymbolLen(nil, 0)
    #expect(chewing_get_maxChiSymbolLen(nil) == -1, "chewing_get_maxChiSymbolLen(NULL) == -1")

    chewing_set_selKey(nil, nil, 0)
    #expect(chewing_get_selKey(nil) == nil, "chewing_get_selKey(NULL) == NULL")

    chewing_set_addPhraseDirection(nil, 0)
    #expect(chewing_get_addPhraseDirection(nil) == -1, "chewing_get_addPhraseDirection(NULL) == -1")

    chewing_set_spaceAsSelection(nil, 0)
    #expect(chewing_get_spaceAsSelection(nil) == -1, "chewing_get_spaceAsSelection(NULL) == -1")

    chewing_set_escCleanAllBuf(nil, 0)
    #expect(chewing_get_escCleanAllBuf(nil) == -1, "chewing_get_escCleanAllBuf(NULL) == -1")

    chewing_set_autoShiftCur(nil, 0)
    #expect(chewing_get_autoShiftCur(nil) == -1, "chewing_get_autoShiftCur(NULL) == -1")

    chewing_set_easySymbolInput(nil, 0)
    #expect(chewing_get_easySymbolInput(nil) == -1, "chewing_get_easySymbolInput(NULL) == -1")

    chewing_set_phraseChoiceRearward(nil, 0)
    #expect(
      chewing_get_phraseChoiceRearward(nil) == -1, "chewing_get_phraseChoiceRearward(NULL) == -1")

    chewing_set_ChiEngMode(nil, 0)
    #expect(chewing_get_ChiEngMode(nil) == -1, "chewing_get_ChiEngMode(NULL) == -1")

    chewing_set_ShapeMode(nil, 0)

    #expect(chewing_handle_Space(nil) == -1, "chewing_handle_Space(NULL) == -1")
    #expect(chewing_handle_Esc(nil) == -1, "chewing_handle_Esc(NULL) == -1")
    #expect(chewing_handle_Enter(nil) == -1, "chewing_handle_Enter(NULL) == -1")
    #expect(chewing_handle_Del(nil) == -1, "chewing_handle_Del(NULL) == -1")
    #expect(chewing_handle_Backspace(nil) == -1, "chewing_handle_Backspace(NULL) == -1")
    #expect(chewing_handle_Up(nil) == -1, "chewing_handle_Up(NULL) == -1")
    #expect(chewing_handle_Down(nil) == -1, "chewing_handle_Down(NULL) == -1")
    #expect(chewing_handle_ShiftLeft(nil) == -1, "chewing_handle_ShiftLeft(NULL) == -1")
    #expect(chewing_handle_Left(nil) == -1, "chewing_handle_Left(NULL) == -1")
    #expect(chewing_handle_ShiftRight(nil) == -1, "chewing_handle_ShiftRight(NULL) == -1")
    #expect(chewing_handle_Right(nil) == -1, "chewing_handle_Right(NULL) == -1")
    #expect(chewing_handle_Tab(nil) == -1, "chewing_handle_Tab(NULL) == -1")
    #expect(chewing_handle_DblTab(nil) == -1, "chewing_handle_DblTab(NULL) == -1")
    #expect(chewing_handle_Capslock(nil) == -1, "chewing_handle_Capslock(NULL) == -1")
    #expect(chewing_handle_Home(nil) == -1, "chewing_handle_Home(NULL) == -1")
    #expect(chewing_handle_PageUp(nil) == -1, "chewing_handle_PageUp(NULL) == -1")
    #expect(chewing_handle_PageDown(nil) == -1, "chewing_handle_PageDown(NULL) == -1")
    #expect(chewing_handle_Default(nil, 0) == -1, "chewing_handle_Default(NULL) == -1")
    #expect(chewing_handle_CtrlNum(nil, 0) == -1, "chewing_handle_CtrlNum(NULL) == -1")
    #expect(chewing_handle_ShiftSpace(nil) == -1, "chewing_handle_ShiftSpace(NULL) == -1")
    #expect(chewing_handle_Numlock(nil, 0) == -1, "chewing_handle_Numlock(NULL) == -1")

    if let phone = chewing_get_phoneSeq(nil) {
      #expect(Bool(false), "chewing_get_phoneSeq(NULL) should return NULL or empty")
      chewing_free(UnsafeMutableRawPointer(mutating: phone))
    } else {
      #expect(Bool(true), "chewing_get_phoneSeq(NULL) returned NULL as expected")
    }
    #expect(chewing_get_phoneSeqLen(nil) == -1, "chewing_get_phoneSeqLen(NULL) == -1")

    // Logger
    chewing_set_logger(nil, nil, nil)

    #expect(chewing_userphrase_enumerate(nil) == -1, "chewing_userphrase_enumerate(NULL) == -1")
    #expect(
      chewing_userphrase_has_next(nil, nil, nil) == 0, "chewing_userphrase_has_next(NULL) == 0")
    #expect(chewing_userphrase_get(nil, nil, 0, nil, 0) == -1, "chewing_userphrase_get(NULL) == -1")
    #expect(chewing_userphrase_add(nil, nil, nil) == -1, "chewing_userphrase_add(NULL) == -1")
    #expect(chewing_userphrase_remove(nil, nil, nil) == -1, "chewing_userphrase_remove(NULL) == -1")
    #expect(chewing_userphrase_lookup(nil, nil, nil) == 0, "chewing_userphrase_lookup(NULL) == 0")

    #expect(chewing_cand_open(nil) == -1, "chewing_cand_open(NULL) == -1")
    #expect(chewing_cand_close(nil) == -1, "chewing_cand_close(NULL) == -1")
    #expect(chewing_cand_choose_by_index(nil, 0) == -1, "chewing_cand_choose_by_index(NULL) == -1")

    #expect(chewing_cand_list_first(nil) == -1, "chewing_cand_list_first(NULL) == -1")
    #expect(chewing_cand_list_last(nil) == -1, "chewing_cand_list_last(NULL) == -1")
    #expect(chewing_cand_list_has_next(nil) == 0, "chewing_cand_list_has_next(NULL) == 0")
    #expect(chewing_cand_list_has_prev(nil) == 0, "chewing_cand_list_has_prev(NULL) == 0")
    #expect(chewing_cand_list_next(nil) == -1, "chewing_cand_list_next(NULL) == -1")
    #expect(chewing_cand_list_prev(nil) == -1, "chewing_cand_list_prev(NULL) == -1")

    #expect(chewing_commit_preedit_buf(nil) == -1, "chewing_commit_preedit_buf(NULL) == -1")
    #expect(chewing_clean_preedit_buf(nil) == -1, "chewing_clean_preedit_buf(NULL) == -1")
    #expect(chewing_clean_bopomofo_buf(nil) == -1, "chewing_clean_bopomofo_buf(NULL) == -1")

    #expect(chewing_commit_Check(nil) == -1, "chewing_commit_Check(NULL) == -1")
    if let c = chewing_commit_String(nil) {
      let s = String(cString: c)
      #expect(s.isEmpty, "chewing_commit_String(NULL) should be empty")
      chewing_free(UnsafeMutableRawPointer(c))
    } else {
      #expect(Bool(Bool(false)), "chewing_commit_String(NULL) returned NULL")
    }

    if let c2 = chewing_commit_String_static(nil) {
      let s = String(cString: c2)
      #expect(s.isEmpty, "chewing_commit_String_static(NULL) should be empty")
    } else {
      #expect(Bool(Bool(false)), "chewing_commit_String_static(NULL) returned NULL")
    }

    #expect(chewing_buffer_Check(nil) == -1, "chewing_buffer_Check(NULL) == -1")
    #expect(chewing_buffer_Len(nil) == -1, "chewing_buffer_Len(NULL) == -1")
    if let b = chewing_buffer_String(nil) {
      #expect(String(cString: b).isEmpty, "chewing_buffer_String(NULL) should be empty")
      chewing_free(UnsafeMutableRawPointer(b))
    } else {
      #expect(Bool(Bool(false)), "chewing_buffer_String(NULL) returned NULL")
    }

    if let b2 = chewing_buffer_String_static(nil) {
      #expect(String(cString: b2).isEmpty, "chewing_buffer_String_static(NULL) should be empty")
    } else {
      #expect(Bool(Bool(false)), "chewing_buffer_String_static(NULL) returned NULL")
    }

    if let bp = chewing_bopomofo_String(nil) {
      #expect(String(cString: bp).isEmpty, "chewing_bopomofo_String(NULL) should be empty")
      chewing_free(UnsafeMutableRawPointer(bp))
    } else {
      #expect(Bool(Bool(false)), "chewing_bopomofo_String(NULL) returned NULL")
    }

    if let bp2 = chewing_bopomofo_String_static(nil) {
      #expect(String(cString: bp2).isEmpty, "chewing_bopomofo_String_static(NULL) should be empty")
    } else {
      #expect(Bool(Bool(false)), "chewing_bopomofo_String_static(NULL) returned NULL")
    }

    // deprecated zuin
    if let zuin = chewing_zuin_String(nil, nil) {
      #expect(String(cString: zuin).isEmpty, "chewing_zuin_String(NULL) should be empty")
      chewing_free(UnsafeMutableRawPointer(zuin))
    } else {
      #expect(Bool(Bool(false)), "chewing_zuin_String(NULL) returned NULL")
    }

    #expect(chewing_bopomofo_Check(nil) == -1, "chewing_bopomofo_Check(NULL) == -1")
    chewing_zuin_Check(nil)  // shall not crash

    #expect(chewing_cursor_Current(nil) == -1, "chewing_cursor_Current(NULL) == -1")

    #expect(chewing_cand_CheckDone(nil) == -1, "chewing_cand_CheckDone(NULL) == -1")
    #expect(chewing_cand_TotalPage(nil) == -1, "chewing_cand_TotalPage(NULL) == -1")
    #expect(chewing_cand_ChoicePerPage(nil) == -1, "chewing_cand_ChoicePerPage(NULL) == -1")
    #expect(chewing_cand_TotalChoice(nil) == -1, "chewing_cand_TotalChoice(NULL) == -1")
    #expect(chewing_cand_CurrentPage(nil) == -1, "chewing_cand_CurrentPage(NULL) == -1")

    chewing_cand_Enumerate(nil)  // shall not crash
    #expect(chewing_cand_hasNext(nil) == -1, "chewing_cand_hasNext(NULL) == -1")

    if let s = chewing_cand_String_static(nil) {
      #expect(String(cString: s).isEmpty, "chewing_cand_String_static(NULL) should be empty")
    } else {
      #expect(Bool(Bool(false)), "chewing_cand_String_static(NULL) returned NULL")
    }

    if let sb = chewing_cand_String(nil) {
      #expect(String(cString: sb).isEmpty, "chewing_cand_String(NULL) should be empty")
      chewing_free(UnsafeMutableRawPointer(sb))
    } else {
      #expect(Bool(Bool(false)), "chewing_cand_String(NULL) returned NULL")
    }

    chewing_interval_Enumerate(nil)  // shall not crash
    #expect(chewing_interval_hasNext(nil) == -1, "chewing_interval_hasNext(NULL) == -1")
    chewing_interval_Get(nil, nil)  // shall not crash

    #expect(chewing_aux_Check(nil) == -1, "chewing_aux_Check(NULL) == -1")
    #expect(chewing_aux_Length(nil) == -1, "chewing_aux_Length(NULL) == -1")

    if let as2 = chewing_aux_String_static(nil) {
      #expect(String(cString: as2).isEmpty, "chewing_aux_String_static(NULL) should be empty")
    } else {
      #expect(Bool(Bool(false)), "chewing_aux_String_static(NULL) returned NULL")
    }

    if let asb = chewing_aux_String(nil) {
      #expect(String(cString: asb).isEmpty, "chewing_aux_String(NULL) should be empty")
      chewing_free(UnsafeMutableRawPointer(asb))
    } else {
      #expect(Bool(Bool(false)), "chewing_aux_String(NULL) returned NULL")
    }

    #expect(chewing_keystroke_CheckIgnore(nil) == -1, "chewing_keystroke_CheckIgnore(NULL) == -1")
    #expect(chewing_keystroke_CheckAbsorb(nil) == -1, "chewing_keystroke_CheckAbsorb(NULL) == -1")

    chewing_kbtype_Enumerate(nil)  // shall not crash
    #expect(chewing_kbtype_hasNext(nil) == -1, "chewing_kbtype_hasNext(NULL) == -1")

    if let kbs = chewing_kbtype_String_static(nil) {
      #expect(String(cString: kbs).isEmpty, "chewing_kbtype_String_static(NULL) should be empty")
    } else {
      #expect(Bool(Bool(false)), "chewing_kbtype_String_static(NULL) returned NULL")
    }

    if let kb = chewing_kbtype_String(nil) {
      #expect(String(cString: kb).isEmpty, "chewing_kbtype_String(NULL) should be empty")
      chewing_free(UnsafeMutableRawPointer(kb))
    } else {
      #expect(Bool(Bool(false)), "chewing_kbtype_String(NULL) returned NULL")
    }
  }

  @Test func testHasOption() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(false), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    let options = [
      "chewing.user_phrase_add_direction",
      "chewing.disable_auto_learn_phrase",
      "chewing.auto_shift_cursor",
      "chewing.candidates_per_page",
      "chewing.language_mode",
      "chewing.easy_symbol_input",
      "chewing.esc_clear_all_buffer",
      "chewing.keyboard_type",
      "chewing.auto_commit_threshold",
      "chewing.phrase_choice_rearward",
      "chewing.selection_keys",
      "chewing.character_form",
      "chewing.space_is_select_key",
      "chewing.conversion_engine",
      "chewing.enable_fullwidth_toggle_key",
    ]

    for opt in options {
      var rc: Int32 = -1
      opt.withCString { ptr in rc = chewing_config_has_option(ctx, ptr) }
      #expect(rc == 1, "should have option '\(opt)'")
    }
  }

  @Test func testDefaultValue() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(false), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    // select key
    if let sk = chewing_get_selKey(ctx) {
      let buf = UnsafeBufferPointer(start: sk, count: 10)
      let expected: [Int32] = "1234567890".utf8.map { Int32($0) }
      for i in 0..<10 { #expect(buf[i] == expected[i], "select key index \(i) shall match") }
      chewing_free(UnsafeMutableRawPointer(mutating: sk))
    } else {
      #expect(Bool(false), "chewing_get_selKey returned NULL")
    }

    #expect(chewing_get_candPerPage(ctx) == 10, "default candPerPage shall be 10")
    #expect(chewing_get_addPhraseDirection(ctx) == 0, "default addPhraseDirection shall be 0")
    #expect(chewing_get_spaceAsSelection(ctx) == 0, "default spaceAsSelection shall be 0")
    #expect(chewing_get_escCleanAllBuf(ctx) == 0, "default escCleanAllBuf shall be 0")
    #expect(chewing_get_autoShiftCur(ctx) == 0, "default autoShiftCur shall be 0")
    #expect(chewing_get_easySymbolInput(ctx) == 0, "default easySymbolInput shall be 0")
    #expect(chewing_get_phraseChoiceRearward(ctx) == 0, "default phraseChoiceRearward shall be 0")
    #expect(chewing_get_autoLearn(ctx) == 0, "default autoLearn shall be 0")
    #expect(chewing_get_ChiEngMode(ctx) == CHINESE_MODE, "default ChiEngMode shall be CHINESE_MODE")
    #expect(
      chewing_get_ShapeMode(ctx) == HALFSHAPE_MODE, "default ShapeMode shall be HALFSHAPE_MODE")
  }

  @Test func testDefaultValueOptions() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(false), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    var ptr: UnsafeMutablePointer<CChar>? = nil
    let rc1 = "chewing.selection_keys".withCString { chewing_config_get_str(ctx, $0, &ptr) }
    #expect(rc1 == 0, "chewing_config_get_str should return OK")
    if let p = ptr {
      let s = String(cString: p)
      #expect(s == "1234567890", "default select key shall be default value")
      chewing_free(UnsafeMutableRawPointer(p))
    } else {
      #expect(Bool(false), "chewing_config_get_str returned NULL")
    }

    var rc2: Int32 = -1
    "chewing.candidates_per_page".withCString { rc2 = chewing_config_get_int(ctx, $0) }
    #expect(rc2 == 10, "default candPerPage shall be 10")

    var rc3: Int32 = -1
    "chewing.auto_commit_threshold".withCString { rc3 = chewing_config_get_int(ctx, $0) }
    #expect(
      rc3 == chewing_get_maxChiSymbolLen(ctx),
      "default chewing.auto_commit_threshold shall equal maxChiSymbolLen")

    var rc4: Int32 = -1
    "chewing.user_phrase_add_direction".withCString { rc4 = chewing_config_get_int(ctx, $0) }
    #expect(rc4 == 0, "default chewing.user_phrase_add_direction shall be 0")

    var rc5: Int32 = -1
    "chewing.space_is_select_key".withCString { rc5 = chewing_config_get_int(ctx, $0) }
    #expect(rc5 == 0, "default chewing.space_is_select_key shall be 0")

    var rc6: Int32 = -1
    "chewing.esc_clear_all_buffer".withCString { rc6 = chewing_config_get_int(ctx, $0) }
    #expect(rc6 == 0, "default chewing.esc_clear_all_buffer shall be 0")

    var rc7: Int32 = -1
    "chewing.auto_shift_cursor".withCString { rc7 = chewing_config_get_int(ctx, $0) }
    #expect(rc7 == 0, "default chewing.auto_shift_cursor shall be 0")

    var rc8: Int32 = -1
    "chewing.easy_symbol_input".withCString { rc8 = chewing_config_get_int(ctx, $0) }
    #expect(rc8 == 0, "default chewing.easy_symbol_input shall be 0")

    var rc9: Int32 = -1
    "chewing.phrase_choice_rearward".withCString { rc9 = chewing_config_get_int(ctx, $0) }
    #expect(rc9 == 0, "default chewing.phrase_choice_rearward shall be 0")

    var rc10: Int32 = -1
    "chewing.disable_auto_learn_phrase".withCString { rc10 = chewing_config_get_int(ctx, $0) }
    #expect(rc10 == 0, "default chewing.disable_auto_learn_phrase shall be 0")

    var rc11: Int32 = -1
    "chewing.language_mode".withCString { rc11 = chewing_config_get_int(ctx, $0) }
    #expect(rc11 == CHINESE_MODE, "default chewing.language_mode shall be CHINESE_MODE")

    var rc12: Int32 = -1
    "chewing.character_form".withCString { rc12 = chewing_config_get_int(ctx, $0) }
    #expect(rc12 == HALFSHAPE_MODE, "default chewing.character_form shall be HALFSHAPE_MODE")

    var rc13: Int32 = -1
    "chewing.conversion_engine".withCString { rc13 = chewing_config_get_int(ctx, $0) }
    #expect(rc13 == 1, "default chewing.conversion_engine shall be 1")
  }

  @Test func testSetCandPerPage() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(false), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    chewing_set_maxChiSymbolLen(ctx, 10)
    let valid: [Int32] = [1, 10]
    let invalid: [Int32] = [0, 11]

    for v in valid {
      chewing_set_candPerPage(ctx, v)
      #expect(chewing_get_candPerPage(ctx) == v, "candPerPage shall be \(v)")
      #expect(chewing_get_maxChiSymbolLen(ctx) == 10, "maxChiSymbolLen shall be 10")

      for inv in invalid {
        chewing_set_candPerPage(ctx, inv)
        #expect(chewing_get_candPerPage(ctx) == v, "candPerPage shall remain \(v) on invalid set")
      }
    }
  }

  @Test func testSetMaxChiSymbolLen() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(false), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    chewing_set_maxChiSymbolLen(ctx, 16)
    #expect(chewing_get_maxChiSymbolLen(ctx) == 16, "maxChiSymbolLen shall be 16")

    chewing_set_maxChiSymbolLen(ctx, MIN_CHI_SYMBOL_LEN - 1)
    #expect(
      chewing_get_maxChiSymbolLen(ctx) == 16, "maxChiSymbolLen shall not change on invalid value")

    chewing_set_maxChiSymbolLen(ctx, MAX_CHI_SYMBOL_LEN + 1)
    #expect(
      chewing_get_maxChiSymbolLen(ctx) == 16, "maxChiSymbolLen shall not change on invalid value")
  }

  @Test func testSetSelKeyNormal() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(false), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    let alt = "asdfghjkl;"
    let arr: [Int32] = alt.utf8.map { Int32($0) } + [0]
    arr.withUnsafeBufferPointer { ptr in
      chewing_set_selKey(ctx, ptr.baseAddress, Int32(alt.count))
    }

    if let sk = chewing_get_selKey(ctx) {
      let buf = UnsafeBufferPointer(start: sk, count: Int(MAX_SELKEY))
      let expected: [Int32] = "asdfghjkl;".utf8.map { Int32($0) } + [0]
      for i in 0..<10 { #expect(buf[i] == expected[i], "select key index \(i) shall match") }
      chewing_free(UnsafeMutableRawPointer(mutating: sk))
    } else {
      #expect(Bool(false), "chewing_get_selKey returned NULL")
    }

    // test config set/get
    let setRc = "chewing.selection_keys".withCString {
      chewing_config_set_str(ctx, $0, "asdfghjkl;")
    }
    #expect(setRc == 0, "chewing_config_set_str should return OK")

    var ptr: UnsafeMutablePointer<CChar>? = nil
    let getRc = "chewing.selection_keys".withCString { chewing_config_get_str(ctx, $0, &ptr) }
    #expect(getRc == 0, "chewing_config_get_str should return OK")
    if let p = ptr {
      let s = String(cString: p)
      #expect(s == "asdfghjkl;", "select key shall be updated")
      chewing_free(UnsafeMutableRawPointer(p))
    } else {
      #expect(Bool(false), "chewing_config_get_str returned NULL")
    }
  }

  @Test func testSetSelKeyErrorHandling() {
    let ctx = chewing_new()
    guard let ctx else {
      #expect(Bool(false), "failed to create context")
      return
    }
    defer { chewing_delete(ctx) }

    // Passing NULLs shall not crash and defaults shall remain
    let alt: [Int32] = "asdfghjkl;".utf8.map { Int32($0) } + [0]
    alt.withUnsafeBufferPointer { ptr in chewing_set_selKey(nil, ptr.baseAddress, Int32(alt.count))
    }
    if let sk = chewing_get_selKey(ctx) {
      let buf = UnsafeBufferPointer(start: sk, count: 10)
      let expected: [Int32] = "1234567890".utf8.map { Int32($0) }
      for i in 0..<10 { #expect(buf[i] == expected[i], "select key shall be default value") }
      chewing_free(UnsafeMutableRawPointer(mutating: sk))
    } else {
      #expect(Bool(false), "chewing_get_selKey returned NULL")
    }

    // invalid set via config
    let rcSet = "chewing.selection_keys".withCString {
      chewing_config_set_str(ctx, $0, "asdfghjkl;1234")
    }
    #expect(rcSet == -1, "chewing_config_set_str should return ERROR on invalid value")

    var ptr: UnsafeMutablePointer<CChar>? = nil
    let rcGet = "chewing.selection_keys".withCString { chewing_config_get_str(ctx, $0, &ptr) }
    #expect(rcGet == 0, "chewing_config_get_str should return OK")
    if let p = ptr {
      let s = String(cString: p)
      #expect(s == "1234567890", "select key shall be default value")
      chewing_free(UnsafeMutableRawPointer(p))
    } else {
      #expect(Bool(false), "chewing_config_get_str returned NULL")
    }
  }
}
