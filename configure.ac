# Process this file with autoconf to produce a configure script.

AC_PREREQ([2.67])

AC_INIT([libchewing],[0.3.5],[chewing-devel@googlegroups.com])
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
LIBCHEWING_MINOR=1

# increment any time the source changes
# Set to 0 if MAJOR or MINOR is increased.
LIBCHEWING_REVISION=0

AC_SUBST(LIBCHEWING_MAJOR)
AC_SUBST(LIBCHEWING_MINOR)
AC_SUBST(LIBCHEWING_REVISION)

# Define a string for the earliest version that this release has
# binary compatibility with. This is used for module locations.
#
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
LT_INIT([win32-dll])

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
AM_CFLAGS="$CFLAGS -Wall"
AS_IF([test x$ac_cv_func_asprintf == xyes],
      [AM_CPPFLAGS="$AM_CPPFLAGS -D_GNU_SOURCE"])

AX_WITH_CURSES
AM_CONDITIONAL([ENABLE_TEXT_UI], [test x$ax_cv_ncursesw = "xyes"])

# Options
dnl Enable gcov for coverage test
AC_ARG_ENABLE([gcov],
              [AS_HELP_STRING([--enable-gcov], [Turn on gcov support @<:@default=no@:>@])],
              [AS_CASE([${enableval}], [yes], [ENABLE_GCOV="true"], [ENABLE_GCOV="false"])],
              [ENABLE_GCOV="false"])
AS_IF([test x$ENABLE_GCOV = x"true"], [AM_CFLAGS="$AM_CFLAGS --coverage"])

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

dnl binary form of chewing data
AC_ARG_ENABLE([binary-data],
                [AS_HELP_STRING([--enable-binary-data],
                                [Experimental use of binary data @<:@default=yes@:>@])],
                [case "${enableval}" in
                no)
                binary_data="no"
                ;;
                *)
                binary_data="yes"
                ;;
                esac],binary_data="yes")
if test x$binary_data = "xyes"; then
        AC_DEFINE(USE_BINARY_DATA, 1, [Experimental use of binary data])
fi
AC_SUBST(ENABLE_BINARY_DATA)
AM_CONDITIONAL(ENABLE_BINARY_DATA, test x$binary_data = "xyes")

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
AC_SUBST(AM_CPPFLAGS)

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
])
AC_OUTPUT

AC_MSG_RESULT([
Build options:
  Build OS                $build_os
  Host OS                 $host_os
  Version                 $PACKAGE_VERSION
  Install prefix          $prefix
  Enable gcov             $ENABLE_GCOV
  Enable binary data      $binary_data
  Build TextUI sample     $ax_cv_ncursesw
  Default CFLAGS          $AM_CFLAGS
])
