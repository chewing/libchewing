	           _                   _
	       ___| |__   _____      _(_)_ __   __ _
	      / __| '_ \ / _ \ \ /\ / / | '_ \ / _` |
	     | (__| | | |  __/\ V  V /| | | | | (_| |
	      \___|_| |_|\___| \_/\_/ |_|_| |_|\__, |
	                                       |___/
	               http://chewing.im/

# libchewing - The intelligent phonetic input method library

The Chewing (酷音) is an intelligent phonetic input method (Zhuyin/Bopomofo)
and is one of the most popular choices for Traditional Chinese users. Chewing
was inspired by other proprietary intelligent Zhuyin input methods on Microsoft
Windows, namely Wang-Xin by Eten, Microsoft New Zhuyin, and Nature Zhuyin (aka
Going). The Chewing developer maintains the project as a fully open-source
effort, positioning it as a leading libre intelligent phonetic solution among
major operating environments.

+ Website: <http://chewing.im/>
+ Issue tracker: <https://github.com/chewing/libchewing/issues>
+ Mailing lists:
   - Development: <https://groups.google.com/group/chewing-devel>
   - General: <https://groups.google.com/group/chewing>
+ Matrix:
   - Development: <https://matrix.to/#/#libchewing:matrix.org>
   - General: <https://matrix.to/#/#chewing-users:matrix.org>
+ Build Status:
   - Github Actions: [![CI](https://github.com/chewing/libchewing/actions/workflows/ci.yml/badge.svg)](https://github.com/chewing/libchewing/actions/workflows/ci.yml)
   - Coverity Scan: [![Coverity Scan Build Status](https://scan.coverity.com/projects/1273/badge.svg)](https://scan.coverity.com/projects/1273)
   - Codecov: [![codecov](https://codecov.io/gh/chewing/libchewing/graph/badge.svg?token=r1piKsG5uF)](https://codecov.io/gh/chewing/libchewing)

libchewing releases can be verified with the following [minisign][] public key

    RWRzJFnXiLZleAyCIv1talBjyRewelcy9gzYQq9pd3SKSFBPoy57sf5s

[minisign]: https://jedisct1.github.io/minisign/

## Status

### 1. System bridge integration

Chewing has been integrated into various input frameworks in Unix-like systems
and even in Microsoft Windows. On these systems, the Chewing package is
typically divided into two parts: libchewing, which manages the actual
character selection logic, and an input framework interface for display and
preference settings.

+ The active integration:
  - [PIME](https://github.com/EasyIME/PIME)
  - [SCIM](https://github.com/chewing/scim-chewing)
  - standalone Microsoft Windows 32/64-bit (windows-chewing)
  - Text Services Framework for Microsoft Windows
  - [ibus](https://github.com/definite/ibus-chewing)
  - [HIME](https://hime-ime.github.io/)
  - [fcitx](https://github.com/fcitx/fcitx5-chewing)
+ The inactive one:
  - mozc
  - [uim](https://github.com/uim/uim-chewing)
  - [ucimf](https://github.com/matlinuxer2/ucimf)
  - JMCCE
  - xcin
  - IIIMF
  - MacOS X (SpaceChewing)
  - Sun's Java Desktop System Input Method Framework
  - OpenVanilla Input Method Framework (previous than version 1.0)
  - OXIM


### 2. support phonetic keyboard layout

  - Hsu
  - IBM
  - Gin-Yieh
  - Eten
  - Eten 26 keys
  - Dvorak
  - Dvorak Hsu
  - HanYu PinYin
  - Taiwan Huayu Luomapinyin
  - MPS2 Pinyin


### 3. External and unmerged projects

libchewing provides straightforward API and design, which enables third-party
projects to deploy the innovative.  Here are the examples:

  * FreeArray: utilizing libchewing for selecting phrases of Array input
   method.
   <https://github.com/shaform/libfreearray>
   <https://github.com/shaform/ibus-freearray>

  * [CitC](https://code.google.com/p/citc/) (Chewing in the Cloud): extending cloud services for real-time
   training and phrase selection based on Chewing IM.

  * [KindleChewing: Chewing IM for Kindle DX device](https://github.com/tjwei/KindleChewing)

  * [NTNU-Master Input Method](http://rportal.lib.ntnu.edu.tw/items/42d5cd11-5fab-4d27-9f26-d01b80588a82): A Faster Chinese IM based on windows-chewing


## Minimal Build Tools Requirement

The following tools are used to build libchewing. Not all tools are necessary
during building. For example, if the compiler you used is clang, gcc & Visual
Studio are not needed. The versions listed here is the minimal version known to
build libchewing. If any tools you use below this version, libchewing might not
be built.

+ Build tools:
   - cmake >= 3.21.0
+ Toolchain / IDE:
   - clang >= 3.2
   - gcc >= 4.6.3
   - Rust >= 1.70
   - Visual Studio Express 2012
+ Documentation tools:
   - texinfo >= 4.12


## Installation

    cmake --preset c99-release
    cmake --build out/build/c99-release
    cmake --install out/build/c99-release --prefix /usr

For macOS:

    brew install libchewing  # latest release version
    brew install --HEAD libchewing  # development, git master branch


## Cross-build

Define a [cmake-toolchains][] file to cross-compile.

Example cross-build instructions:

    cmake --preset c99-release --toolchain arm-none-linux-gnueabi.cmake
    cmake --build out/build/c99-release

[cmake-toolchains]: https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html

## Build on Windows with MinGW

To build libchewing on Windows, you need to setup MinGW and MSYS in your
system. The installer of MinGW and MSYS is in the following link:

<https://sourceforge.net/projects/mingw/files/Installer/mingw-get-inst/>

In "Select Components" during installing, please select the following items:

- MinGW Compiler Suite -> C Compiler
- MSYS Basic System

After installing, execute [MinGW directory]\msys\1.0\msys.bat (default is
C:\MinGW\msys\1.0\msys.bat) to enter MSYS shell.

Now you have the build environment for libchewing. You can follow the installation
steps to build with cmake.

## Build on OS X

To build libchewing on OS X, you will need tools listed in the requirement.
Since OS X does not ship with those tools, building those tools from source
could be a tricky task.

A simple way to install those tools is by using Homebrew, a package manager
for OS X. You can learn more about Homebrew or see the installation
instruction from

http://brew.sh

Once Homebrew is installed, run the following commands to install the tools
you need:

    brew install cmake
    brew install rustup
    brew install texinfo


### cmake

Because OS X uses an older version of `makeinfo`, you have to set MAKEINFO
manually to where Homebrew installed makeinfo. For example:

        # cmake -DMAKEINFO=/usr/local/Cellar/texinfo/5.2/bin/makeinfo .

then

        # make


## Usage

( modified from <https://code.google.com/p/ibus/wiki/ChewingUserGuide> )

Chewing guides the user to input Chinese by its pronunciation, in the form of
either [Bopomofo/Zhuyin][1] or [Hanyu pinyin][2], as well as Chinese punctuation
marks and normal and full shape number and English alphabets.

The following sections assume you are using the default configuration, that is,
with default/DaChen Bopomofo keyboard layout, on an en_US keyboard, and default
key-binding.


### Glossary

Preedit buffer: The places for storing your typing before sending to the
applications (such as firefox) you are using.

Mode: Mode determines how Chewing reacts on keyboard input.


### Editing mode

This mode is for normal Chinese character and punctuation typing. You are more
likely working on this mode.

In this mode, alpha-numberic and punctuation marks are interpreted as either
Bopomofo symbols or punctuation marks. If the symbols can form Chinese
characters, it will choose the most suitable character according to the
context in preedit buffer.

It is recommended that you enter the whole Chinese sentences, so Chewing can
do auto-correction for you.  If you like what you see, you can press Enter to
commit the characters in preedit buffer.

If something goes wrong, you can select the character by move your cursor
using {Left} or {Right}, then press {Down} to enter Candidate Selection mode
to choose words.

To prevent the auto-correction on certain phrase, you may press {Tab} to
break the auto-correction on the end of the sentence.

You can also remember the 2,3,4 words phrases by pressing {Ctrl-2}, {Ctrl-3},
and {Ctrl-4} on the end of the phrase.

Also note that Shift's behavior changes in this mode. Shift-(alpha number)
outputs corresponding Full shape Chinese symbols if "Easy symbol input" is
enabled; Or outputs corresponding number half shape lowercase English
alphabets if "Easy symbol input" is disabled.

Talking about inputing Chinese symbols, other then enable "Easy symbol input"
mode, you can also press {Ctrl-1} or {`} to open up a symbol selection dialog,
select the category, then use {Down} key to choose the symbols as you would do
for characters.


	Key binding   API name                   Functionality
	-----------   --------                   -------------
	Caps Lock     chewing_handle_Capslock    Toggle Temporary English sub-mode
	Down          chewing_handle_Down        Enter Candidate Selection mode
	Shift-Space   chewing_handle_ShiftSpace  Toggle Half/Full Shape sub-mode
	Enter         chewing_handle_Enter       Commit the content in preedit buffer
	                                         to active application window
	Tab           chewing_handle_Tab         Break the auto-correction.
	Ctrl-1        chewing_handle_CtrlNum     Open symbol selection dialog
	Ctrl-2        chewing_handle_CtrlNum     Remember 2-word phrase.
	Ctrl-3        chewing_handle_CtrlNum     Remember 3-word phrase.
	Ctrl-4        chewing_handle_CtrlNum     Remember 4-word phrase.


### Half/Full Shape sub-mode

This sub-mode is for inputing half shape and full shape characters. Half shape
characters are essentially normal English characters; while the full shape
characters are full-width pretty symbols that resemble English characters.

	Key binding   API name                   Functionality
	-----------   --------                   -------------
	Shift-Space   chewing_handle_ShiftSpace  Toggle Half/Full Shape sub-mode


### Temporary English sub-mode

This sub-mode is for temporary English inputting.

	Key binding   API name                   Functionality
	-----------   --------                   -------------
	Caps Lock     chewing_handle_Capslock    Toggle Temporary English sub-mode


### Candidate Selection mode

This mode is for Choosing the candidate. Firstly, the longest phrases that fit
the pronunciation are shown, then the second shortest, till the single
characters. Pressing {Down} again to return to the longest phrases.

For example, after entering "w91o3g4". Pressing {Down} makes Chewing displays
3-word candidate "台北市"; pressing {Down} again makes Chewing displays 2-word
candidate "北市"; pressing {Down} again makes Chewing displays 1-word
candidates "市","是"; pressing {Down} again makes Chewing displays back the
3-word candidates "台北市";

	Key binding   API name                   Functionality
	-----------   --------                   -------------
	Down          chewing_handle_Down        Next bunch of candidates in
	                                         different length
	Left          chewing_handle_Left        Previous page of candidates
	Right         chewing_handle_Right       Next page of candidates
	1, 2, ...0    chewing_handle_Default     Select 1st, 2nd, ... 10th candidate


### Bypass mode

This mode is on whenever the preedit buffer is empty. This allows you to use
movement keys (such as cursor keys and page up/page down) and popular key
binding (Ctrl-A, Ctrl-S).

For the brief usage of libchewing APIs, please check the simplified example
implemented in file contrib/simple-select.c

[1]: https://en.wikipedia.org/wiki/Bopomofo
[2]: https://en.wikipedia.org/wiki/Pinyin


## History

Libchewing is derived from the original Chewing input method,
a module of XCIN that focuses on intelligent phonetic (Bopomofo/Zhuyin) processing and was initially intended for use with the X Window System.
This input method module was developed by Lu-chuan Kung (lckung) and Kang-pen Chen (kpchen),
and was sponsored by Tsan-sheng Hsu from Academia Sinica between 1999 and 2001.
However, the original authors eventually ceased the development of Chewing, and its strong coupling with XCIN limited its application in broader contexts.
Additionally, there was a similar input method, bimsphone, which was included in the XCIN server.
Like Chewing, bimsphone also lacked a convenient API for further development.
In 2002, Jim Huang, along with others, formed the Chewing core team and extended the work of Kung and Chen.
The Chewing core team renamed the project "New Chewing" to differentiate their work from the original.
Nevertheless, the English name has remained "Chewing," which is recognized by various input method frameworks as well.


## License

Except the following source code:

* thirdparty/sqlite-amalgamation/ contains sqlite3 source which is in public
  domain. See <https://www.sqlite.org/copyright.html> for more information.

* cmake/FindCurses.cmake is modified from CMake source, which is licensed under
   BSD 3-Clause.

All source code are licensed under GNU LGPL v2.1 (Lesser General Public License
v2.1). See "COPYING" for details.


## Authors & Contact Information

See "AUTHORS" for details.
