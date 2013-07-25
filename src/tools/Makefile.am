INCLUDES = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src \
	-I$(top_srcdir)/src/porting_layer/include \
	$(NULL)

CC = $(CC_FOR_BUILD)
AM_CFLAGS = $(CFLAGS_FOR_BUILD)

noinst_PROGRAMS = sort maketree

sort_SOURCES = \
	sort.c \
	$(top_builddir)/src/common/key2pho.c \
	$(top_builddir)/src/common/chewing-utf8-util.c \
	$(NULL)

maketree_SOURCES = maketree.c
