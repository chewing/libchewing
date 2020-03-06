SUBDIRS = common porting_layer

AM_CPPFLAGS = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	-DCHEWING_DATADIR=\"$(datadir)/libchewing\" \
	$(USERPHRASE_CPPFLAGS) \
	$(DEFAULT_CPPFLAGS) \
	$(NULL)

noinst_HEADERS = \
	private.h \
	$(NULL)

lib_LTLIBRARIES = libchewing.la
libchewing_la_SOURCES = \
	compat.c \
	chewingio.c \
	chewingutil.c \
	choice.c \
	dict.c \
	tree.c \
	bopomofo.c \
	pinyin.c \
	mod_aux.c \
	userphrase.c \
	$(USERPHRASE_SOURCES) \
	$(NULL)

libchewing_la_LIBADD = \
	$(top_builddir)/src/common/libcommon.la \
	$(top_builddir)/src/porting_layer/src/libporting_layer.la \
	$(NULL)

libchewing_la_LDFLAGS = \
	-version-number $(LIBCHEWING_MAJOR):$(LIBCHEWING_MINOR):$(LIBCHEWING_REVISION) \
	-rpath $(libdir) \
	-no-undefined \
	$(NO_UNDEFINED_LDFLAGS) \
	-export-symbols-regex "^(chewing)_" \
	$(NULL)

if WITH_SQLITE3
libchewing_la_SOURCES += \
	chewing-sql.c \
	userphrase-sql.c \
	$(NULL)

if WITH_INTERNAL_SQLITE3
AM_CPPFLAGS += -I$(top_srcdir)/thirdparty/sqlite-amalgamation
libchewing_la_LIBADD += \
	$(top_builddir)/thirdparty/sqlite-amalgamation/libsqlite3-internal.la \
	$(NULL)
endif

else
libchewing_la_SOURCES += \
	hash.c \
	userphrase-hash.c \
	$(NULL)
endif
