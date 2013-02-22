tooldir = $(top_builddir)/src/tools
if ENABLE_BINARY_DATA
chindexs = \
	ch_index_begin.dat \
	ch_index_phone.dat \
	$(NULL)
else
chindexs = ch_index.dat
endif
datas = \
	us_freq.dat \
	dict.dat \
	ph_index.dat \
	fonetree.dat \
	$(chindexs) \
	$(NULL)
static_tables = pinyin.tab swkb.dat symbols.dat
generated_header = $(top_builddir)/src/chewing-definition.h

dist_noinst_DATA = \
	NOTE \
	phone.cin \
	phone.cin-CNS11643-complete.patch \
	svnrev \
	tsi.src \
	$(NULL)

chewing_datadir = $(pkglibdir)
chewing_data_DATA = \
	$(static_tables) \
	$(datas) \
	$(NULL)

all: $(datas)

$(datas): gendata_stamp

gendata_stamp: phone.cin tsi.src
	-if test -f $(generated_header); then \
		echo "chewing-definition.h exists."; \
	fi
	$(MAKE) gendata && \
	touch $@

gendata:
	env LC_ALL=C $(tooldir)/sort$(EXEEXT) $(top_srcdir)/data/phone.cin $(top_srcdir)/data/tsi.src
	$(tooldir)/maketree$(EXEEXT)
	-rm -f phoneid.dic
	-mv -f chewing-definition.h $(generated_header)

CLEANFILES = $(datas) gendata_stamp $(generated_header)
