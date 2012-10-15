# Process this file with autoconf to produce a configure script.

AC_PREREQ(2.59)

AC_INIT(libchewing, 0.3.3, chewing-devel@googlegroups.com)
AC_CONFIG_AUX_DIR([autotools])
AC_CONFIG_SRCDIR([src/chewingio.c])
AC_CONFIG_MACRO_DIR([m4])

AC_SUBST(PACKAGE_VERSION)

# libtool versioning for libchewing

# increment if the interface has additions, changes, removals.
LIBCHEWING_CURRENT=3

# increment any time the source changes; set to 0 if you increment CURRENT
LIBCHEWING_REVISION=1

# increment if any interfaces have been added; set to 0
# if any interfaces have been removed. removal has
# precedence over adding, so set to 0 if both happened.
LIBCHEWING_AGE=0

AC_SUBST(LIBCHEWING_CURRENT)
AC_SUBST(LIBCHEWING_REVISION)
AC_SUBST(LIBCHEWING_AGE)

# Define a string for the earliest version that this release has
# binary compatibility with. This is used for module locations.
#
LIBCHEWING_BINARY_VERSION=1.0.0
AC_SUBST(LIBCHEWING_BINARY_VERSION)

AC_DEFINE_UNQUOTED(LIBCHEWING_BINARY_VERSION, "$LIBCHEWING_BINARY_VERSION", [The binary version of libchewing.])
AC_DEFINE_UNQUOTED(LIBCHEWING_VERSION, "$PACKAGE_VERSION", [The release version of libchewing.])

# Init automake stuff
AM_MAINTAINER_MODE([enable])
AM_INIT_AUTOMAKE
AM_SILENT_RULES([yes])
AM_CONFIG_HEADER([include/config.h])
AC_PREFIX_DEFAULT(/usr)

# Init libtool
AC_LIBTOOL_DLOPEN
AC_PROG_LIBTOOL
AC_SUBST(LIBTOOL_DEPS)
AC_SUBST(LIBTOOL_EXPORT_OPTIONS)

# libtool option to control which symbols are exported
# right now, symbols starting with _ are not exported
LIBTOOL_EXPORT_OPTIONS='-export-symbols-regex "^[[^_]].*"'
AC_SUBST(LIBTOOL_EXPORT_OPTIONS)

# Checks for programs.
AC_PROG_CC
AC_LANG(C)
AC_C_CONST

# plat_mmap_posix
AC_CHECK_FUNCS([getpagesize])
AC_CHECK_FUNCS([munmap])
AC_CHECK_HEADERS([fcntl.h])
AC_FUNC_MMAP

# gen_keystroke
AC_CHECK_FUNCS([setenv])
AC_CHECK_FUNCS([setlocale])
AC_CHECK_HEADERS([locale.h])

# chewingutil.c
AC_CHECK_FUNCS([strcspn])

# mod_aux.c
AC_CHECK_FUNCS([strdup])

# hash.c
AC_CHECK_HEADERS([limits.h])

# chewing-utf8-util.h
AC_CHECK_HEADERS([wchar.h])
AC_TYPE_SIZE_T

CC_FOR_BUILD=${CC_FOR_BUILD-${CC}}
AC_SUBST(CC_FOR_BUILD)

# Default CFLAGS
AM_CFLAGS="$CFLAGS -Wall"

# Checks if doxygen is avaiable
AC_PATH_PROG(DOXYGEN, doxygen, no)
AM_CONDITIONAL(HAVE_DOXYGEN, test x$DOXYGEN != xno)
AC_SUBST(DOXYGEN)

# Checks if graphviz is available
AC_CHECK_PROG(HAVE_GRAPHVIZ, dot, YES, NO)
AC_PATH_PROG(DOT, dot)
AC_SUBST(DOT)
AC_PATH_PROG(PERL, perl)
AC_SUBST(PERL)

# Checks for libraries.
AC_HEADER_STDC
AC_CHECK_HEADERS([stdlib.h string.h unistd.h])

# Checks for typedefs, structures, and compiler characteristics.
AC_C_CONST

# Checks for library functions.
AC_FUNC_MALLOC
AC_FUNC_MEMCMP
AC_CHECK_FUNCS([memchr memmove memset mkdir strchr strstr])

# optional modules
CHECK_REQUIRED_VERSION=0.9.4
NCURSESW_REQUIRED_VERSION=5.9
NCURSES_REQUIRED_VERSION=5.9

# Checks for check unit test framework
PKG_CHECK_MODULES(CHECK, check >= $CHECK_REQUIRED_VERSION,
                  enable_check=yes, enable_check=no)
AM_CONDITIONAL(ENABLE_UNIT_TEST, test x$enable_check = "xyes")

# Checks for ncursesw
PKG_CHECK_MODULES(NCURSESW, ncursesw >= $NCURSESW_REQUIRED_VERSION,
                  enable_ncursesw=yes, enable_ncursesw=no)
AM_CONDITIONAL(ENABLE_TEXT_UI, test x$enable_ncursesw = "xyes")
if test "$enable_ncursesw" = "yes"; then
    AC_DEFINE(HAVE_CURSES_H, 1, [ Define to 1 if you have the <curses.h> header file. ])
fi

# Options

dnl Debugging switch
AC_ARG_ENABLE([debug],
              [AS_HELP_STRING([--enable-debug],
                              [Turn on debugging support @<:@default=no@:>@])],
              [case "${enableval}" in
               yes)
               LIBDEBUG="true"
               ;;
               *)
               ;;
               esac],
              [LIBDEBUG="false"])

if test x$LIBDEBUG = x"true"; then
        AC_DEFINE(ENABLE_DEBUG, 1,
                [Define to 1 if you want native library runtime debugging code enabled])
fi
AC_SUBST(LIBDEBUG)

dnl Enable gcov for coverage test
AC_ARG_ENABLE([gcov],
              AS_HELP_STRING([--enable-gcov], [Turn on gcov support @<:@default=no@:>@]),
              AS_CASE([${enableval}], [yes], [ENABLE_GCOV="true"], [ENABLE_GCOV="false"]),
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

AC_OUTPUT([
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

AC_MSG_RESULT([
Build options:
  Build OS                $build_os
  Host OS                 $host_os
  Version                 $PACKAGE_VERSION
  Install prefix          $prefix
  Enable debug            $LIBDEBUG
  Enable gcov             $ENABLE_GCOV
  Enable binary data      $binary_data
  Build Unit Test         $enable_check
  Build TextUI sample     $enable_ncursesw
  Default CFLAGS          $AM_CFLAGS
])
