MAINTAINERCLEANFILES = Makefile.in

AM_CPPFLAGS = \
	-DLIBDIR=\"@libdir@\" \
	$(NULL)

INCLUDES = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	$(NULL)

noinst_LTLIBRARIES = libporting_layer.la
libporting_layer_la_SOURCES = \
	plat_mmap_posix.c \
	plat_mmap_windows.c \
	plat_path.c \
	$(NULL)
