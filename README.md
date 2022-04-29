	           _                   _
	       ___| |__   _____      _(_)_ __   __ _
	      / __| '_ \ / _ \ \ /\ / / | '_ \ / _` |
	     | (__| | | |  __/\ V  V /| | | | | (_| |
	      \___|_| |_|\___| \_/\_/ |_|_| |_|\__, |
	                                       |___/
	               http://chewing.im/

# libchewing - The intelligent phonetic input method library

The Chewing (酷音) is an intelligent phonetic (Zhuyin/Bopomofo) input method,
one of the most popular choices for Traditional Chinese users. Chewing was
inspired by other proprietary intelligent Zhuyin input methods under Microsoft
Windows, namely, Wang-Xin by Eten, Microsoft New Zhuyin, and Nature Zhuyin.

The Chewing core team extended their work and actively maintains the project
as full open source efforts.

+ Website: <http://chewing.im/>
+ Issue tracker: <https://github.com/chewing/libchewing/issues>
+ Mailing lists:
   - Development: <https://groups.google.com/group/chewing-devel>
   - General: <https://groups.google.com/group/chewing>
+ Matrix:
   - Development: <https://matrix.to/#/#libchewing:matrix.org>
   - General: <https://matrix.to/#/#chewing-users:matrix.org>
+ Build Status:
   - Travis-Ci: [![Status-Icon](https://travis-ci.org/chewing/libchewing.svg?branch=master)](https://travis-ci.org/chewing/libchewing)
   - Coverity Scan: [![Coverity Scan Build Status](https://scan.coverity.com/projects/1273/badge.svg)](https://scan.coverity.com/projects/1273)
   - Coveralls: [![Coverage Status](https://img.shields.io/coveralls/chewing/libchewing.svg)](https://coveralls.io/r/chewing/libchewing?branch=master)


## History

libchewing is derived from the original Chewing input method, a xcin module
focusing on intelligent phonetic (Bopomofo/Zhuyin) processing by Lu-chuan
Kung (lckung) and Kang-pen Chen (kpchen) sponsored by Tsan-sheng Hsu from
Academia Sinica during 1999-2001.  However, the original authors of Chewing
dropped its development, and Chewing was highly coupled with xcin, which
prevents from comprehensive applications.  There was a similar input method,
bimsphone, which was bundled in XCIN server. However, it did not provide a
convenient API for further development, either.


## Motivation

Jim Huang, et al. formed the Chewing core team and extended Kung and Chen's
work since 2002.  The chewing core team renamed the project as "new" chewing
to differentiate their work from the original.  Nevertheless, the English name
has remained "chewing", which is identified by various input method framework
as well.

In 2004, Chewing core team successfully ported the input method to several
platforms and framework along with community collaboration.


## Development Goal of libchewing

- Split logic and view.
- Support multiple operating systems, and input framework.
- Provide a universal API for input framework and further development.


## Status


### 1. System bridge integration

Chewing has been adopted by various input frameworks in Unix-like systems and
even Microsoft Windows.  On these systems, the chewing package is usually
split into two parts: libchewing, which handles the actual character selection
logic; and input framework interface for display and preference setting.

+ The active integration:
  - JMCCE
  - SCIM
  - standalone Microsoft Windows 32/64-bit (windows-chewing)
  - Text Services Framework for Microsoft Windows
  - UIM
  - ibus
  - UCIMF
  - mozc
  - gcin/HIME
  - fcitx
+ The inactive one:
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

  * CitC (Chewing in the Cloud): extending cloud services for real-time
   training and phrase selection based on Chewing IM.
   <https://code.google.com/p/citc/>

  * KindleChewing: Chewing IM for Kindle DX device
   <https://github.com/tjwei/KindleChewing>

  * NTNU-Master Input Method: A Faster Chinese IM based on windows-chewing
   <https://sites.google.com/site/ntnumaster001/>


## Minimal Build Tools Requirement

The following tools are used to build libchewing. Not all tools are necessary
during building. For example, if the compiler you used is clang, gcc & Visual
Studio are not needed. The versions listed here is the minimal version known to
build libchewing. If any tools you use below this version, libchewing might not
be built.

+ Build tools:
   - autoconf >= 2.65
   - automake >= 1.11.6
   - libtool >= 2.4.2
   - cmake >= 2.8.8 (optional)
+ Toolchain / IDE:
   - clang >= 3.2
   - gcc >= 4.6.3
   - Visual Studio Express 2012
+ Documentation tools:
   - texinfo >= 4.12


## Continuous Integration (CI) Service

libchewing uses the following CI services:

- <https://travis-ci.org/chewing/libchewing> (testing)
- <https://drone.io/github.com/chewing/libchewing> (testing)
- <https://coveralls.io/r/chewing/libchewing> (coverage report)
- <https://launchpad.net/~chewing/+archive/chewing> (ubuntu PPA)


## Installation

	# ./configure --prefix=/usr
	  (If you checkout from GIT, make sure running ./autogen.sh
	   before this.)
	# make
	# make install

For macOS:

	# brew install libchewing  # latest release version
	# brew install --HEAD libchewing  # development, git master branch

see "INSTALL" for details.


## Cross-build

Example cross-build instructions:

	# CC_FOR_BUILD=gcc ./configure \
	    --host=arm-none-linux-gnueabi \
	    --disable-shared --enable-static
	# make


## Build on Windows with MinGW

To build libchewing on Windows, you need to setup MinGW and MSYS in your
system. The installer of MinGW and MSYS is in the following link:

<https://sourceforge.net/projects/mingw/files/Installer/mingw-get-inst/>

In "Select Components" during installing, please select the following items:

- MinGW Compiler Suite -> C Compiler
- MSYS Basic System

After installing, execute <MinGW directory>\msys\1.0\msys.bat (default is
C:\MinGW\msys\1.0\msys.bat) to enter MSYS shell.

If you get the source from the git repository you need additional step
to make the source buildable; use the following commands to install
necessary packages.

	- mingw-get install automake
	- mingw-get install autoconf
	- mingw-get install libtool

Now you have the build environment for libchewing. However, you need to check
the line end of source code is LF instead of CR/LF before running autogen.sh.
The easily way to do this is using git:

	- git config core.autocrlf input
	- rm -rf *
	- git reset --hard

Now you can run the following commands in MSYS shell to build libchewing:

If you get the source from the git repository, run:

	- autogen.sh

Then

	- configure
	- make

If you run testchewing.exe (available after `make check') you will
find that testchewing.exe cannot print the correct string. This is
because testchewing.exe prints the UTF-8 string, while Windows cannot
print it to console.


## Build with CMake

libchewing supports cmake (<https://www.cmake.org/>) build system. You can use the
following command to build with cmake:

	- cmake .
	- make

cmake is also the preferred way to build libchewing on Windows platform due to
better Windows integrated. You can use the following command to create Visual
Studio project in 32-bits Windows platform:

	- cmake . -G "Visual Studio 11"

or the following command to create Visual Studio project in 64-bits Windows
platform:

	- cmake . -G "Visual Studio 11 Win64" (64-bits Windows)


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

        # brew install autoconf automake
        # brew install libtool
        # brew install cmake

        # brew install texinfo


### Autotools (autoconf, automake)

If you get the source from the git repository, run:

        # ./autogen.sh

Because OS X uses an older version of `makeinfo`, you have to set MAKEINFO
manually to where Homebrew installed makeinfo. For example:

        # ./configure MAKEINFO=/usr/local/Cellar/texinfo/5.2/bin/makeinfo

then

        # make


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
