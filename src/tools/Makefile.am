INCLUDES = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src \
	-I$(top_srcdir)/src/porting_layer/include

CC = $(CC_FOR_BUILD)
AM_CFLAGS = $(CFLAGS_FOR_BUILD)

noinst_PROGRAMS = sort_word sort_dic maketree

sort_word_SOURCES = sort_word.c \
		    $(top_builddir)/src/common/key2pho.c \
		    $(top_builddir)/src/common/chewing-utf8-util.c

sort_dic_SOURCES = sort_dic.c \
		   $(top_builddir)/src/common/key2pho.c \
		   $(top_builddir)/src/common/chewing-utf8-util.c

maketree_SOURCES = maketree.c
