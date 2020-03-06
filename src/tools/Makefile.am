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

noinst_PROGRAMS = init_database dump_database

init_database_SOURCES = \
	init_database.c \
	$(top_srcdir)/src/common/chewing-utf8-util.c \
	$(top_srcdir)/src/common/key2pho.c \
	$(NULL)

dump_database_SOURCES = \
	dump_database.c \
	$(top_srcdir)/src/common/chewing-utf8-util.c \
	$(top_srcdir)/src/common/key2pho.c \
	$(top_srcdir)/src/porting_layer/src/plat_mmap_posix.c \
	$(top_srcdir)/src/porting_layer/src/plat_mmap_windows.c \
	$(top_srcdir)/src/porting_layer/src/rpl_malloc.c \
	$(NULL)
