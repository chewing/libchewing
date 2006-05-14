INCLUDES = \
	-I$(top_builddir)/include \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/src \
	-I$(top_srcdir)/src/porting_layer/include

noinst_PROGRAMS = sort_word sort_dic maketree $(DAT2BIN)

#if ENABLE_BINARY_DATA
#DAT2BIN = dat2bin
#dat2bin_SOURCE = dat2bin.c
#dat2bin_LDADD = $(top_builddir)/src/common/libcommon.la
#endif

sort_word_LDADD = $(top_builddir)/src/common/libcommon.la
sort_word_SOURCES = sort_word.c

sort_dic_LDADD =  $(top_builddir)/src/common/libcommon.la
sort_dic_SOURCES = sort_dic.c

maketree_SOURCES = maketree.c
