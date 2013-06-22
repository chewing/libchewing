SUBDIRS = common porting_layer
INCLUDES = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	-DCHEWING_DATADIR=\"$(datadir)/chewing\" \
	$(NULL)

noinst_HEADERS = \
	private.h \
	$(NULL)

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
	pinyin.c \
	mod_aux.c \
	$(NULL)

libchewing_la_LIBADD = \
	$(top_builddir)/src/common/libcommon.la \
	$(top_builddir)/src/porting_layer/src/libporting_layer.la \
	$(NULL)

libchewing_la_LDFLAGS = \
	-version-number $(LIBCHEWING_MAJOR):$(LIBCHEWING_MINOR):$(LIBCHEWING_REVISION) \
	-rpath $(libdir) \
	-no-undefined \
	-Wl,--no-undefined \
	-export-symbols-regex "^(chewing)_" \
	$(NULL)
