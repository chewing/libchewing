AUX_DIST = \
	$(ac_aux_dir)/config.guess \
	$(ac_aux_dir)/config.sub \
	$(ac_aux_dir)/compile \
	$(ac_aux_dir)/depcomp \
	$(ac_aux_dir)/install-sh \
	$(ac_aux_dir)/ltconfig \
	$(ac_aux_dir)/ltmain.sh \
	$(ac_aux_dir)/missing

EXTRA_DIST = \
	config.rpath
	mkinstalldirs \
	autogen.sh \
	chewing.pc \
	libchewing.spec \
	Doxyfile

AUTOMAKE_OPTIONS = gnu
SUBDIRS = src/common src/tools data src

MAINTAINERCLEANFILES = \
	Makefile.in \
	aclocal.m4 \
	configure \
	config.h.in \
	$(AUX_DIST)

DISTCLEANFILES = chewing.pc

CLEANFILES = *.bak 

pkgconfigdir = $(libdir)/pkgconfig

pkgconfig_DATA = chewing.pc

chewing_headers = \
	include/char.h \
	include/chewingio.h \
	include/dict.h \
	include/hanyupinyin.h \
	include/mod_aux.h \
	include/userphrase.h \
	include/chewing.h \
	include/chewing-utf8-util.h \
	include/global.h \
	include/hash.h \
	include/zuin.h

install-data-local:
	$(mkinstalldirs) $(DESTDIR)/$(prefix)/include/chewing
	$(INSTALL_DATA) $(chewing_headers) $(DESTDIR)/$(prefix)/include/chewing
install-manpages:
	$(mkinstalldirs) $(DESTDIR)/$(prefix)/share/man/man3
	$(INSTALL_DATA) man/man3/*.3 $(DESTDIR)/$(prefix)/share/man/man3
docs:
	if [ -x "@DOXYGEN@" ]; then \
		@DOXYGEN@ Doxyfile; \
	fi
clean-local:
	rm -rf html
	rm -rf man
