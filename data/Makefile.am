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

all: checkdata_stamp $(datas)

$(datas): gendata_stamp

checkdata_stamp: phone.cin tsi.src
	@echo "timestamp" > $@

LOCALE_RESET= \
export LC_ALL=C

gendata_stamp:
	-if test -f $(generated_header); then \
		echo "chewing-definition.h exists."; \
	else \
		$(MAKE) gendata; \
	fi
	@echo "timestamp" > $@

gendata:
	$(tooldir)/sort_word$(EXEEXT) $(top_srcdir)/data/phone.cin
	$(shell $(LOCALE_RESET) ; $(tooldir)/sort_dic$(EXEEXT) $(top_srcdir)/data/tsi.src)
	$(tooldir)/maketree$(EXEEXT)
	-rm -f phoneid.dic
	-mv -f chewing-definition.h $(top_builddir)/src/

CLEANFILES = $(datas) gendata_stamp checkdata_stamp $(generated_header)
