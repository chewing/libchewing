```
	           _                   _
	       ___| |__   _____      _(_)_ __   __ _
	      / __| '_ \ / _ \ \ /\ / / | '_ \ / _` |
	     | (__| | | |  __/\ V  V /| | | | | (_| |
	      \___|_| |_|\___| \_/\_/ |_|_| |_|\__, |
	                                       |___/
	               https://chewing.im/
```

# libchewing - The intelligent phonetic input method library

The Chewing (酷音) is an intelligent phonetic input method (Zhuyin/Bopomofo)
and is one of the most popular choices for Traditional Chinese users. Chewing
was inspired by other proprietary intelligent Zhuyin input methods on Microsoft
Windows, namely Wang-Xin by Eten, Microsoft New Zhuyin, and Nature Zhuyin (aka
Going). The Chewing developer maintains the project as a fully open-source
effort, positioning it as a leading libre intelligent phonetic solution among
major operating environments.

+ Website: <https://chewing.im/>
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

libchewing releases can be verified with the following OpenPGP public key

&emsp;[083B3CAB64267E5BAB7159673EF0C673DADCC30C][pgp_key] Libchewing Signing Key &lt;release@chewing.im&gt;

or the following [minisign][] public key

&emsp;RWRzJFnXiLZleAyCIv1talBjyRewelcy9gzYQq9pd3SKSFBPoy57sf5s

[pgp_key]: https://chewing.im/.well-known/openpgpkey/hu/y84sdmnksfqswe7fxf5mzjg53tbdz8f5?l=release
[minisign]: https://jedisct1.github.io/minisign/

## Status

### 1. System bridge integration

Chewing has been integrated into various input frameworks in Unix-like systems
and even in Microsoft Windows and Android. On these systems, the Chewing package
is typically divided into two parts: libchewing, which manages the actual
character selection logic, and an input framework interface for display and
preference settings.

+ The active integration:
  - [Windows TSF](https://github.com/chewing/windows-chewing-tsf)
  - [PIME](https://github.com/EasyIME/PIME)
  - [ibus](https://github.com/chewing/ibus-chewing)
  - [HIME](https://hime-ime.github.io/)
  - [fcitx](https://github.com/fcitx/fcitx5-chewing)
  - [Guileless Bopomofo](https://github.com/hiroshiyui/GuilelessBopomofo)
+ The inactive one: [SCIM](https://github.com/chewing/scim-chewing), standalone Microsoft Windows 32/64-bit ([windows-chewing](https://github.com/chewing/windows-chewing)), mozc, [uim](https://github.com/uim/uim-chewing), [ucimf](https://github.com/matlinuxer2/ucimf), JMCCE, xcin, IIIMF, standalone MacOS X (SpaceChewing), Sun's Java Desktop System Input Method Framework, OpenVanilla Input Method Framework (previous than version 1.0), and OXIM.


### 2. support phonetic keyboard layout

  - DaChen (default)
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
  - Colemak-DH ANSI
  - Colemak-DH Ortholinear


### 3. External and unmerged projects

libchewing provides a straightforward API and design, enabling third-party
projects to deploy innovative features. Here are some examples:
* [FreeArray](https://github.com/shaform/libfreearray): utilizing libchewing for
  selecting phrases of Array input method.
* [CitC](https://code.google.com/p/citc/) (Chewing in the Cloud): extending cloud
  services for real-time training and phrase selection based on Chewing IM.
* [KindleChewing: Chewing IM for Kindle DX device](https://github.com/tjwei/KindleChewing)
* [NTNU-Master Input Method](http://rportal.lib.ntnu.edu.tw/items/42d5cd11-5fab-4d27-9f26-d01b80588a82): A faster Chinese IM based on windows-chewing


## Build and Installation

### Prerequisites

The following tools are used to build libchewing. Not all tools are necessary
during building. For example, if the compiler you used is clang, gcc & Visual
Studio are not needed. The versions listed here is the minimal version known to
build libchewing. If any tools you use below this version, libchewing might not
be built.

+ Build tools:
   - cmake >= 3.21.0
+ Toolchain / IDE:
   - clang >= 3.2 OR gcc >= 4.6.3
   - Rust >= 1.83
   - Build Tools for Visual Studio 2022 for MSVC build
   - Vcpkg for MSVC build
+ Documentation tools:
   - texinfo >= 4.8


### Build via CMake

Use the default preset:

    cmake --preset default --install-prefix /usr
    cmake --build build
    cmake --build build -t test
    cmake --build build -t install

Build the rust implementation:

    cmake --preset rust-release --install-prefix /usr
    cmake --build build
    cmake --build build -t test
    cmake --build build -t install

Check other supported presets:

    cmake --list-presets

### Cross-build

Define a [cmake-toolchains][] file to cross-compile.

Example cross-build instructions:

    cmake --preset default --toolchain arm-none-linux-gnueabi.cmake
    cmake --build build

[cmake-toolchains]: https://cmake.org/cmake/help/latest/manual/cmake-toolchains.7.html

### Build on Windows with Build Tools for Visual Studio 2022

To build libchewing on Windows and link to other program build from MSVC, you
need to use the MSVC toolchain. To install the build environment:

Open admin prompt `cmd.exe`

    winget install Microsoft.VisualStudio.2022.BuildTools
    winget install Ninja-build.Ninja
    winget install Kitware.CMake
    winget install Rustlang.Rustup


Optional development tools

    winget install Git.Git
    winget install VSCodium.VSCodium

Reboot, then open `Visual Studio Installer` and install C/C++ components.

Open `x64 Native Tools Command Prompt for VS 2022`

    rustup default stable
    cmake -G Ninja --preset rust

Now you have the build environment for libchewing. You can follow the installation
steps to build with cmake.

### Build on macOS

To build libchewing on macOS, you will need tools listed in the requirements.
Since macOS does not ship with these tools, building them from source can be
a tricky task.

A simple way to install these tools is through [Homebrew](https://brew.sh/),
a package manager for macOS. Once Homebrew is installed, run the following
commands to install the tools you need:

    brew install cmake
    brew install rustup
    rustup default stable

### Minimum Supported Rust Version

To ensure libchewing can be built on various Linux distributions, we use the
minimum rust version available from major distributions' next release branch.
Data source: https://repology.org/project/rust/versions

* Current MSRV: 1.83.0 (Debian unstable)


## Usage

Chewing enables users to input Chinese by its pronunciation, using either
[Bopomofo/Zhuyin][1] or [Hanyu pinyin][2]. It also supports Chinese punctuation
marks, as well as both normal and full-shape numbers and the English alphabet.

The following sections are based on the assumption that you are using the
default configuration. This includes the default/DaChen Bopomofo keyboard layout
on an en\_US keyboard, along with the default key-binding.

### Glossary

Preedit Buffer: This is the area where your typing is stored before being sent
to the applications (such as Firefox) you are using.

Mode: This determines how Chewing responds to keyboard input.

### Editing mode

This mode facilitates the typing of normal Chinese characters and punctuation
and is typically the default working mode.

In this mode, alphanumeric characters and punctuation marks are interpreted as
Bopomofo symbols or punctuation marks. When these symbols form Chinese
characters, the system chooses the most appropriate character based on the
context in the preedit buffer.

Entering complete Chinese sentences is advantageous as it allows the system to
perform auto-correction. To confirm the output, pressing Enter will commit the
characters in the preedit buffer.

In case of errors, characters can be selected by moving the cursor with {Left}
or {Right}, followed by pressing {Down} to enter Candidate Selection mode for
word choice.

Auto-correction for a specific phrase can be overridden by pressing {Tab} at the
end of the sentence.

Memorization of 2, 3, or 4-word phrases is possible by pressing {Ctrl-2},
{Ctrl-3}, or {Ctrl-4} at the phrase's end.

The behavior of the Shift key changes in this mode. Using Shift with an
alphanumeric key outputs corresponding full-shape Chinese symbols if "Easy
Symbol Input" is enabled, or outputs corresponding half-shape lowercase English
alphabets if "Easy Symbol Input" is disabled.

For inputting Chinese symbols, aside from enabling "Easy Symbol Input" mode,
pressing {Ctrl-1} or {`} opens a symbol selection dialog. After selecting the
category, the {Down} key can be used to choose symbols as one would for
characters.

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

This sub-mode is for inputting half-shape and full-shape characters. Half-shape
characters are essentially normal English characters, while full-shape
characters are stylized symbols that resemble English characters in a larger,
more prominent format.

	Key binding   API name                   Functionality
	-----------   --------                   -------------
	Shift-Space   chewing_handle_ShiftSpace  Toggle Half/Full Shape sub-mode


### Temporary English sub-mode

This sub-mode is for temporary English inputting.

	Key binding   API name                   Functionality
	-----------   --------                   -------------
	Caps Lock     chewing_handle_Capslock    Toggle Temporary English sub-mode


### Candidate Selection mode

This mode is for choosing the candidate. It first displays the longest phrases
that match the pronunciation, followed by progressively shorter phrases, down
to single characters. Pressing {Down} cycles back to the longest phrases.

For example, after entering "w91o3g4" and pressing {Down}, Chewing displays
the 3-word candidate "台北市". Pressing {Down} again shows the 2-word candidate
"北市". Another press of {Down} brings up 1-word candidates "市" and "是".
Pressing {Down} once more cycles back to the 3-word candidate "台北市".


	Key binding   API name                   Functionality
	-----------   --------                   -------------
	Down          chewing_handle_Down        Next bunch of candidates in
	                                         different length
	Left          chewing_handle_Left        Previous page of candidates
	Right         chewing_handle_Right       Next page of candidates
	1, 2, ...0    chewing_handle_Default     Select 1st, 2nd, ... 10th candidate


### Bypass mode

This mode is active whenever the preedit buffer is empty. It enables the use
of movement keys (such as cursor keys and page up/page down) and popular key
bindings (such as Ctrl-A, Ctrl-S).

For a brief overview of using the libchewing APIs, please refer to the
simplified example in the file `contrib/simple-select.c`.

[1]: https://en.wikipedia.org/wiki/Bopomofo
[2]: https://en.wikipedia.org/wiki/Pinyin


## History

Libchewing is derived from the original Chewing input method, a module of XCIN
that focuses on intelligent phonetic (Bopomofo/Zhuyin) processing and was
initially intended for use with the X Window System. This input method module
was developed by Lu-chuan Kung (lckung) and Kang-pen Chen (kpchen), and was
sponsored by Tsan-sheng Hsu from Academia Sinica between 1999 and 2001.

However, the original authors eventually ceased the development of Chewing, and
its strong coupling with XCIN limited its application in broader contexts.
Additionally, there was a similar input method, bimsphone, which was included in
the XCIN server. Like Chewing, bimsphone also lacked a convenient API for
further development. In 2002, Jim Huang, along with others, formed the Chewing
core team and extended the work of Kung and Chen. The Chewing core team renamed
the project "New Chewing" to differentiate their work from the original.
Nevertheless, the English name has remained "Chewing," which is recognized by
various input method frameworks as well.


## License

Except for the following source code, all other source code is licensed under
the GNU LGPL v2.1 (Lesser General Public License v2.1), or (at your option) any
later version. See "COPYING" for details:
* The directory "thirdparty/sqlite-amalgamation" contains the SQLite3 source,
  which is in the public domain. For more information, see <https://www.sqlite.org/copyright.html>.
* The file "cmake/FindCurses.cmake" is modified from the CMake source and is
  licensed under the BSD 3-Clause license.


## Authors & Contact Information

See "AUTHORS" for details.
