MAINTAINERCLEANFILES = Makefile.in

AM_CPPFLAGS = \
	-DCHEWING_DATADIR=\"$(datadir)/libchewing\" \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	$(DEFAULT_CPPFLAGS) \
	$(NULL)

noinst_LTLIBRARIES = libporting_layer.la
libporting_layer_la_SOURCES = \
	plat_mmap_posix.c \
	plat_mmap_windows.c \
	plat_path.c \
	rpl_malloc.c \
	$(NULL)
