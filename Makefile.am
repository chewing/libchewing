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
SUBDIRS = src data

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

install-data-local:
	$(mkinstalldirs) $(DESTDIR)/$(prefix)/include/chewing
	$(INSTALL_DATA) include/*.h $(DESTDIR)/$(prefix)/include/chewing
install-manpages:
	$(mkinstalldirs) $(DESTDIR)/$(prefix)/share/man/man3
	$(INSTALL_DATA) man/man3/*.3 $(DESTDIR)/$(prefix)/share/man/man3
docs:
	if [ -x "@DOXYGEN@" ]; then \
		@DOXYGEN@ Doxyfile; \
	fi
