ACLOCAL_AMFLAGS = -I m4
AUTOMAKE_OPTIONS = foreign

SUBDIRS = src/tools data src test doc

CMAKE_FILES = \
	cmake/config.h.in \
	cmake/FindCurses.cmake \
	cmake/version.texi.in \
	CMakeLists.txt \
	$(NULL)

CONTRIB_FILES = \
	contrib/python/test.py \
	contrib/python/chewing.py \
	contrib/simple-select.c \
	contrib/Makefile \
	$(NULL)

EXTRA_DIST = \
	autogen.sh \
	$(CMAKE_FILES) \
	$(CONTRIB_FILES) \
	$(NULL)

pkgconfigdir = $(libdir)/pkgconfig
pkgconfig_DATA = chewing.pc

chewing_includedir = $(includedir)/chewing
chewing_include_HEADERS = \
	include/chewingio.h \
	include/mod_aux.h \
	include/chewing.h \
	include/chewing-compat.h \
	include/global.h \
	$(NULL)

noinst_HEADERS =\
	include/internal/chewing-private.h \
	include/internal/chewing-sql.h \
	include/internal/chewing-utf8-util.h \
	include/internal/chewingutil.h \
	include/internal/choice-private.h \
	include/internal/dict-private.h \
	include/internal/global-private.h \
	include/internal/hash-private.h \
	include/internal/key2pho-private.h \
	include/internal/memory-private.h \
	include/internal/pinyin-private.h \
	include/internal/tree-private.h \
	include/internal/userphrase-private.h \
	include/internal/bopomofo-private.h \
	thirdparty/sqlite-amalgamation/sqlite3ext.h \
	thirdparty/sqlite-amalgamation/sqlite3.h \
	$(NULL)

dist_noinst_DATA = \
	README.md \
	$(NULL)

# Maintainer's release target
release:
	$(MAKE) distdir
	$(top_srcdir)/scripts/git2cl > $(distdir)/ChangeLog
	cat $(top_srcdir)/scripts/ChangeLog-svn >> $(distdir)/ChangeLog
	tardir=$(distdir) && $(am__tar) | BZIP2=$${BZIP2--9} bzip2 -c >$(distdir).tar.bz2
	$(am__remove_distdir)
