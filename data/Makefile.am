tooldir = $(top_builddir)/src/tools
datas = \
	us_freq.dat \
	dict.dat \
	ph_index.dat \
	fonetree.dat \
	ch_index_begin.dat \
	ch_index_phone.dat \
	$(NULL)
static_tables = pinyin.tab swkb.dat symbols.dat

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
	$(MAKE) gendata && \
	touch $@

gendata:
	env LC_ALL=C $(tooldir)/sort$(EXEEXT) $(top_srcdir)/data/phone.cin $(top_srcdir)/data/tsi.src
	$(tooldir)/maketree$(EXEEXT)
	-rm -f phoneid.dic

CLEANFILES = $(datas) gendata_stamp
