ACLOCAL_AMFLAGS = -I m4

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

AUTOMAKE_OPTIONS = foreign
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

chewing_includedir = $(includedir)/chewing
chewing_include_HEADERS = \
	include/chewingio.h \
	include/mod_aux.h \
	include/chewing.h \
	include/global.h

installdirs:
	$(mkinstalldirs) $(DESTDIR)$(prefix)/share/man/man3

install-manpages: installdirs
	$(INSTALL_DATA) man/man3/*.3 $(DESTDIR)$(prefix)/share/man/man3
docs:
	if [ -x "@DOXYGEN@" ]; then \
		@DOXYGEN@ Doxyfile; \
	fi
clean-local:
	rm -rf html
	rm -rf man
