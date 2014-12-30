AM_CPPFLAGS = \
	$(DEFAULT_CPPFLAGS) \
	$(NULL)

AM_LDFLAGS = -static

if WITH_SQLITE3
if WITH_INTERNAL_SQLITE3
noinst_LTLIBRARIES = libsqlite3-internal.la

libsqlite3_internal_la_SOURCES = \
	sqlite3.c \
	$(NULL)

check_PROGRAMS = sqlite3

sqlite3_SOURCES = \
	shell.c \
	$(NULL)

sqlite3_LDADD = libsqlite3-internal.la
endif
endif
