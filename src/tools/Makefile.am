AM_CPPFLAGS = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src \
	-I$(top_srcdir)/src/porting_layer/include \
	-I$(top_srcdir)/thirdparty/sqlite-amalgamation \
	$(DEFAULT_CPPFLAGS) \
	$(NULL)

CC = $(CC_FOR_BUILD)
AM_CFLAGS = $(CFLAGS_FOR_BUILD)

noinst_PROGRAMS = init_database

init_database_SOURCES = \
	init_database.c \
	$(NULL)
init_database_LDADD = \
	$(top_builddir)/src/common/libcommon.la \
	$(NULL)
