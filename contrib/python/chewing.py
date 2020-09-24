from ctypes import *
from functools import partial
import sys

_libchewing = None
if sys.platform == "win32": # Windows
    import os.path
    # find in current dir first
    dll_path = os.path.join(os.path.dirname(__file__), "chewing.dll")
    if not os.path.exists(dll_path):
        dll_path = "chewing.dll" # search in system path
    _libchewing = CDLL(dll_path)
else: # UNIX-like systems
    _libchewing = CDLL('libchewing.so.3')

_libchewing.chewing_commit_String.restype = c_char_p
_libchewing.chewing_buffer_String.restype = c_char_p
_libchewing.chewing_cand_String.restype = c_char_p
_libchewing.chewing_zuin_String.restype = c_char_p
_libchewing.chewing_aux_String.restype = c_char_p
_libchewing.chewing_get_KBString.restype = c_char_p
_libchewing.chewing_new.restype = c_void_p
_libchewing.chewing_new2.restype = c_void_p


def Init(datadir, userdir):
    return _libchewing.chewing_Init(datadir, userdir)


class ChewingContext:
    def __init__(self, **kwargs):
        if not kwargs:
            self.ctx = _libchewing.chewing_new()
        else:
            syspath = kwargs.get("syspath", None)
            userpath = kwargs.get("userpath", None)
            self.ctx = _libchewing.chewing_new2(
                syspath,
                userpath,
                None,
                None)

    def __del__(self):
        _libchewing.chewing_delete(c_void_p(self.ctx))

    def __getattr__(self, name):
        func = 'chewing_' + name
        if hasattr(_libchewing, func):
            wrap = partial(getattr(_libchewing, func), c_void_p(self.ctx))
            setattr(self, name, wrap)
            return wrap
        else:
            raise AttributeError(name)

    def Configure(self, cpp, maxlen, direction, space, kbtype):
        self.set_candPerPage(cpp)
        self.set_maxChiSymbolLen(maxlen)
        self.set_addPhraseDirection(direction)
        self.set_spaceAsSelection(space)
        self.set_KBType(kbtype)
