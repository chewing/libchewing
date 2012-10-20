ACLOCAL_AMFLAGS = -I m4

AUX_DIST = \
	$(ac_aux_dir)/config.guess \
	$(ac_aux_dir)/config.sub \
	$(ac_aux_dir)/compile \
	$(ac_aux_dir)/depcomp \
	$(ac_aux_dir)/install-sh \
	$(ac_aux_dir)/ltconfig \
	$(ac_aux_dir)/ltmain.sh \
	$(ac_aux_dir)/missing \
	$(NULL)

EXTRA_DIST = \
	autogen.sh \
	chewing.pc \
	libchewing.spec \
	m4/gnulib-cache.m4 \
	$(NULL)

AUTOMAKE_OPTIONS = foreign
SUBDIRS = lib src/tools data src test doc

MAINTAINERCLEANFILES = \
	Makefile.in \
	aclocal.m4 \
	configure \
	config.h.in \
	$(AUX_DIST) \
	$(NULL)

DISTCLEANFILES = chewing.pc

CLEANFILES = *.bak 

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
