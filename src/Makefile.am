SUBDIRS = common tools
INCLUDES = -I$(top_builddir)/include

lib_LTLIBRARIES = libchewing.la
libchewing_la_SOURCES = \
	char.c \
	chewingio.c \
	chewingutil.c \
	choice.c \
	dict.c \
	hash.c \
	tree.c \
	userphrase.c \
	zuin.c \
	hanyupinying.c
libchewing_la_LIBADD = common/libcommon.la
libchewing_la_LDFLAGS = \
	-version-info $(LIBCHEWING_CURRENT):$(LIBCHEWING_REVISION):$(LIBCHEWING_AGE) \
	-export-dynamic \
	@LIBTOOL_EXPORT_OPTIONS@
	-rpath $(libdir)
