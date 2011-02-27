# Process this file with autoconf to produce a configure script.

AC_PREREQ(2.59)

AC_INIT(libchewing, 0.3.3, chewing-devel@googlegroups.com)
AC_CONFIG_AUX_DIR([autotools])
AC_CONFIG_SRCDIR([src/chewingio.c])

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
AM_INIT_AUTOMAKE
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

CC_FOR_BUILD=${CC_FOR_BUILD-${CC}}
AC_SUBST(CC_FOR_BUILD)

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

# Checks for check unit test framework
PKG_CHECK_MODULES(CHECK, check >= $CHECK_REQUIRED_VERSION,
		  enable_check=yes, enble_check=no)
AM_CONDITIONAL(ENABLE_UNIT_TEST, test x$enable_check = "xyes")

# Checks for ncursesw
OLDCFLAGS=$CFLAGS
LIB_NAME=ncursesw
NCURSESW_CFLAGS=
NCURSESW_LIBS=
for p in "$HOME/include" "$prefix/include" /usr/local/include /usr/include
do
	if test -f "$p/ncursesw/ncurses.h"
	then
		NCURSESW_CFLAGS="-I$p"
		AC_DEFINE(HAVE_NCURSESW_NCURSES_H, 1, [ Define to 1 if you have the <ncursesw/ncurses.h> header file. ])
		break
	elif test -f "$p/ncurses/ncurses.h"
	then
		NCURSESW_CFLAGS="-I$p"
		AC_DEFINE(HAVE_NCURSES_NCURSES_H, 1, [ Define to 1 if you have the <ncurses/ncurses.h> header file. ])
		break
	fi
done

for p in "$HOME/lib" "$prefix/lib" /usr/local/lib /usr/lib
do
	if test -f "$p/libncursesw.so"
	then
		if test "$p" != x/usr/lib
		then
			NCURSESW_LIBS="-L$p -lncursesw"
		else
			NCURSESW_LIBS="-lncursesw"
		fi
		break
	fi
done

enable_ncursesw=yes
if test -z "$NCURSESW_CFLAGS"; then
	enable_ncursesw=no
fi
if test -z "$NCURSESW_LIBS"; then
	enable_ncursesw=no
fi
AC_SUBST(NCURSESW_CFLAGS)
AC_SUBST(NCURSESW_LIBS)
AM_CONDITIONAL(ENABLE_TEXT_UI, test x$enable_ncursesw = "xyes")

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
	CFLAGS="$CFLAGS -g"
fi
AC_SUBST(LIBDEBUG)

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
  CFLAGS="$CFLAGS -fvisibility=hidden"
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
win*)
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

AC_OUTPUT([
	Makefile 
	data/Makefile 
	src/Makefile
	src/porting_layer/Makefile
	src/porting_layer/src/Makefile
	src/common/Makefile
	src/tools/Makefile 
	test/Makefile
	chewing.pc 
	libchewing.spec
	Doxyfile
])

AC_MSG_RESULT([
Build options:
  Version                 $PACKAGE_VERSION
  Install prefix          $prefix
  Enable debug            $LIBDEBUG
  Enable binary data      $binary_data
  Build Unit Test         $enable_check
  Build TextUI sample     $enable_ncursesw
])
