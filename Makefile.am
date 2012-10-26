ACLOCAL_AMFLAGS = -I m4
AUTOMAKE_OPTIONS = foreign

SUBDIRS = src/tools data src test doc

EXTRA_DIST = \
	autogen.sh \
	$(NULL)

pkgconfigdir = $(libdir)/pkgconfig
pkgconfig_DATA = chewing.pc

chewing_includedir = $(includedir)/chewing
chewing_include_HEADERS = \
	include/chewingio.h \
	include/mod_aux.h \
	include/chewing.h \
	include/global.h \
	$(NULL)

noinst_HEADERS =\
	include/internal/char-private.h \
	include/internal/chewing-private.h \
	include/internal/chewing-utf8-util.h \
	include/internal/chewingutil.h \
	include/internal/choice-private.h \
	include/internal/dict-private.h \
	include/internal/global-private.h \
	include/internal/hanyupinyin-private.h \
	include/internal/hash-private.h \
	include/internal/key2pho-private.h \
	include/internal/tree-private.h \
	include/internal/userphrase-private.h \
	include/internal/zuin-private.h \
	$(NULL)
