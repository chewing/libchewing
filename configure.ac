# Process this file with autoconf to produce a configure script.

AC_PREREQ([2.65])

# Update CMakeLists.txt if version is changed.
AC_INIT([libchewing],[0.5.1],[chewing-devel@googlegroups.com])
AC_CONFIG_AUX_DIR([build-aux])
AC_CONFIG_SRCDIR([src/chewingio.c])
AC_CONFIG_MACRO_DIR([m4])

AC_SUBST(PACKAGE_VERSION)

# libtool versioning for libchewing

# Update CMakeLists.txt if one of MAJOR/MINOR/REVISION is updated.
#
# We use major/minor/revision instead of current/revision/age format here is due
# to buggy behavior in libtool in BSD family. See the following link for detail
# information.
#
#   https://lists.gnu.org/archive/html/bug-libtool/2011-05/msg00007.html
#
# The following link shows versioning policies in FreeBSD for your reference.
#
#   http://www.freebsd.org/doc/en/books/developers-handbook/policies-shlib.html

# increment if the interface has incompatible update (changes, removals).
LIBCHEWING_MAJOR=3

# increment if the inteface has compatible update (additions).
# Set to 0 if MAJOR is increased.
LIBCHEWING_MINOR=3

# increment any time the source changes
# Set to 0 if MAJOR or MINOR is increased.
LIBCHEWING_REVISION=1

AC_SUBST(LIBCHEWING_MAJOR)
AC_SUBST(LIBCHEWING_MINOR)
AC_SUBST(LIBCHEWING_REVISION)

# Define a string for the earliest version that this release has
# binary compatibility with. This is used for module locations.
#
# Update CMakeFiles.txt if LIBCHEWING_BINARY_VERSION is changed.
LIBCHEWING_BINARY_VERSION=1.0.0
AC_SUBST(LIBCHEWING_BINARY_VERSION)

AC_DEFINE_UNQUOTED(LIBCHEWING_BINARY_VERSION,
        "$LIBCHEWING_BINARY_VERSION", [The binary version of libchewing.])
AC_DEFINE_UNQUOTED(LIBCHEWING_VERSION,
        "$PACKAGE_VERSION", [The release version of libchewing.])

# Init automake stuff
AM_INIT_AUTOMAKE
AM_SILENT_RULES([yes])
AC_CONFIG_HEADERS([include/config.h])

# Init libtool
LT_INIT([win32-dll pic-only])

# libtool option to control which symbols are exported
# right now, symbols starting with _ are not exported
AC_SUBST(LIBTOOL_EXPORT_OPTIONS, ['-export-symbols-regex "^[[^_]].*"'])

# Checks for programs.
AC_PROG_CC
AC_PROG_CC_C99
AM_PROG_CC_C_O
AC_LANG(C)
AC_C_BIGENDIAN

# Checks for library functions.
AC_FUNC_MALLOC
AC_CHECK_FUNCS([strtok_r asprintf])

# options
AC_ARG_WITH([sqlite3],
            AS_HELP_STRING([--with-sqlite3], [Use sqlite3 to store userphrase @<:@default=yes@:>@]),
            [],
            [with_sqlite3=yes])
AM_CONDITIONAL([WITH_SQLITE3], [test x"$with_sqlite3" = x"yes"])

AC_ARG_WITH([internal-sqlite3],
            AS_HELP_STRING([--with-internal-sqlite3], [Use internal sqlite3 instead of system-wide @<:@default=no@:>@]),
            [],
            [with_internal_sqlite3=no])
AM_CONDITIONAL([WITH_INTERNAL_SQLITE3], [test x"$with_internal_sqlite3" = x"yes"])

# for sqlite
AS_IF([test x"$with_sqlite3" = x"yes"], [
       AC_DEFINE([WITH_SQLITE3], [1], [Use sqlite3 to store userphrase])
       AS_IF([test x"$with_internal_sqlite3" = x"no"],
             [
              AC_SEARCH_LIBS([sqlite3_open], [sqlite3],
                             [AS_IF([test x$ac_cv_search_sqlite3_open != x"none required"],
                                    [AM_LDFLAGS="$AM_LDFLAGS $ac_cv_search_sqlite3_open"])],
                                    [AC_MSG_ERROR([unable to find the sqlite3() function])])
             ],
             [
              AC_SEARCH_LIBS([dlopen], [dl dld],
                             [AS_IF([test x$ac_cv_search_dlopen != x"none required"],
                                    [AM_LDFLAGS="$AM_LDFLAGS $ac_cv_search_dlopen"])],
                                    [AC_MSG_ERROR([unable to find the dlopen() function])])
              AX_PTHREAD([
                          AM_CFLAGS="$AM_CFLAGS $PTHREAD_CFLAGS"
                          AM_LDFLAGS="$AM_LDFLAGS $PTHREAD_LIBS"
                          ], [AC_MSG_ERROR([cannot find pthread])])
              ])
])

# plat_mmap_posix
AC_FUNC_MMAP

# chewing-utf8-util.h
AC_TYPE_SIZE_T

# chewing-private.h
AC_C_INLINE
AC_TYPE_UINT16_T

CC_FOR_BUILD=${CC_FOR_BUILD-${CC}}
AC_SUBST(CC_FOR_BUILD)

# Default CFLAGS
AM_CFLAGS="$AM_CFLAGS -Wall $CFLAGS"

# Add '_GNU_SOURCE' for asprintf
AS_IF([test x$ac_cv_func_asprintf = xyes],
      [DEFAULT_CPPFLAGS="$DEFAULT_CPPFLAGS -D_GNU_SOURCE"])

AX_WITH_CURSES
AM_CONDITIONAL([ENABLE_TEXT_UI], [test x$ax_cv_ncursesw = "xyes"])

# Options
dnl Enable gcov for coverage test
AC_ARG_ENABLE([gcov],
              [AS_HELP_STRING([--enable-gcov], [Turn on gcov support @<:@default=no@:>@])],
              [AS_CASE([${enableval}], [yes], [ENABLE_GCOV="true"], [ENABLE_GCOV="false"])],
              [ENABLE_GCOV="false"])
AS_IF([test x$ENABLE_GCOV = x"true"], [AM_CFLAGS="$AM_CFLAGS --coverage" AM_LDFLAGS="$AM_LDFLAGS --coverage"])

dnl Adds -fvisibility=hidden to CFLAGS if running with gcc 4 or greater.
AC_MSG_CHECKING([whether the compiler supports GCC Visibility])
dnl Check for gcc4 or greater
AC_COMPILE_IFELSE([AC_LANG_PROGRAM([[
void
#if defined(__GNUC__) && (__GNUC__ >= 4)
foo () {};
#endif
]], [[]])],[
  has_visibility=yes
  AM_CFLAGS="$AM_CFLAGS -fvisibility=hidden"
],[
  has_visibility=no
])
AC_MSG_RESULT($has_visibility)

AC_CACHE_CHECK([whether the linker supports --no-undefined],
               [ac_cv_no_undefined],
               [
  saved_LDFLAGS="${LDFLAGS}"
  LDFLAGS="${LDFLAGS} -Wl,--no-undefined"
  AC_TRY_LINK([], [], [ac_cv_no_undefined=yes], [ac_cv_no_undefined=no])
  LDFLAGS="${saved_LDFLAGS}"
])
AS_IF([test x$ac_cv_no_undefined = "xyes"], [NO_UNDEFINED_LDFLAGS="-Wl,--no-undefined"])
AC_SUBST(NO_UNDEFINED_LDFLAGS)

# Platform-dependent
dnl What kind of system are we using?
case $host_os in
win*|mingw*)
        SYSTEM=windows
        ;;
cygwin*)
        SYSTEM=unix
        ;;
*)
        SYSTEM=unix
        ;;
esac

case $SYSTEM in
win)
AC_DEFINE(UNDER_WINDOWS, 1,
        [Runtime is under Win32 environment])
        ;;
unix)
AC_DEFINE(UNDER_POSIX, 1,
        [Runtime is under POSIX environment])
        ;;
esac

AC_SUBST(AM_CFLAGS)
AC_SUBST(AM_LDFLAGS)
AC_SUBST(DEFAULT_CPPFLAGS)

AC_CONFIG_LINKS([
        data/pinyin.tab:data/pinyin.tab
        data/swkb.dat:data/swkb.dat
        data/symbols.dat:data/symbols.dat
        test/stresstest.py:test/stresstest.py
])

AC_CONFIG_FILES([
        Makefile
        chewing.pc
        data/Makefile
        doc/Makefile
        libchewing.spec
        src/Makefile
        src/common/Makefile
        src/porting_layer/Makefile
        src/porting_layer/src/Makefile
        src/tools/Makefile
        test/Makefile
        thirdparty/sqlite-amalgamation/Makefile
])
AC_OUTPUT

AC_MSG_RESULT([
Build options:
  Build OS                $build_os
  Host OS                 $host_os
  Version                 $PACKAGE_VERSION
  Install prefix          $prefix
  Enable gcov             $ENABLE_GCOV
  With sqlite3            $with_sqlite3
  With internal sqlite3   $with_internal_sqlite3
  Build TextUI sample     $ax_cv_ncursesw
  Default CFLAGS          $AM_CFLAGS
  Default CPPFLAGS        $DEFAULT_CPPFLAGS
  Default LDFLAGS         $AM_LDFLAGS
])
