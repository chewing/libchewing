//
//  ChewingKey.swift
//  ChewingSimplified
//
//  Created by Rumen Cholakov on 2.06.25.
//

import Foundation
@_implementationOnly import CLibChewing

/// Keys recognized by the Chewing engine.
///
/// This enum maps high‑level key names (enter, space, backspace)
/// to the underlying C constants (`CHEWING_KEY_Enter`, etc.) used by the C API.
public enum ChewingKey {
    /// Enter key (ASCII 10)
    case enter
    /// Space key (ASCII 32)
    case space
    /// Backspace key (ASCII 127)
    case backspace
    
    /// The underlying CChar value that the Chewing C API expects.
    ///
    /// Converts this enum case into the corresponding `CHEWING_KEY_*` constant.
    public var cValue: CChar {
        switch self {
        case .enter:     return CHEWING_KEY_Enter
        case .space:     return CHEWING_KEY_Space
        case .backspace: return CHEWING_KEY_Backspace
        }
    }
    
    /// A Swift `Character` representation of this key.
    ///
    /// Converts the underlying CChar into a `Character` for convenience
    /// (e.g., printing or feeding back into `process(key:)`).
    public var character: Character {
        return Character(UnicodeScalar(UInt8(self.cValue)))
    }
}
