# Process this file with autoconf to produce a configure script.

AC_PREREQ(2.59)

AC_INIT(libchewing, 0.2.6, jserv@kaffe.org)
AC_CONFIG_SRCDIR([src/chewingio.c])

AC_SUBST(PACKAGE_VERSION)

# libtool versioning for libchewing

# increment if the interface has additions, changes, removals.
LIBCHEWING_CURRENT=1

# increment any time the source changes; set to 0 if you increment CURRENT
LIBCHEWING_REVISION=0

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
AC_PREFIX_DEFAULT(/usr)

# Init libtool
AC_LIBTOOL_DLOPEN
AC_PROG_LIBTOOL
AC_LIB_LTDL
AC_SUBST(LIBTOOL_DEPS)

# libtool option to control which symbols are exported
# right now, symbols starting with _ are not exported
LIBTOOL_EXPORT_OPTIONS='-export-symbols-regex "^[[^_]].*"'

# Checks for programs.
AC_PROG_CC
AC_LANG(C)
AC_C_CONST

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

# Options
AC_ARG_ENABLE([debug],
		[AS_HELP_STRING(--enable-debug,Turn on debugging support)],
		[case "${enableval}" in
		yes)
		LIBDEBUG="true"
		AC_DEFINE(DEBUG, 1,
		[Define to 1 if you want native library runtime debugging code enabled])
		CFLAGS="$CFLAGS -g"
		;;
		no)
		LIBDEBUG="false"
		;;
		*)
		AC_MSG_ERROR(bad value ${enableval} for --enable-debug)
		;;
		esac],
	[LIBDEBUG="false"])
AC_SUBST(LIBDEBUG)


AC_OUTPUT([
	Makefile 
	data/Makefile 
	src/Makefile
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
])
