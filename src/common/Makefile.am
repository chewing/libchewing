AM_CPPFLAGS = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	$(DEFAULT_CPPFLAGS) \
	$(NULL)

noinst_LTLIBRARIES = libcommon.la

libcommon_la_SOURCES = \
	key2pho.c \
	chewing-utf8-util.c \
	$(NULL)

if WITH_INTERNAL_SQLITE3
AM_CPPFLAGS += \
	-I$(top_srcdir)/thirdparty/sqlite-amalgamation \
	$(NULL)
endif
