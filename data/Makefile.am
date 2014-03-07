tooldir = $(top_builddir)/src/tools
datas = \
	dictionary.dat \
	index_tree.dat \
	$(NULL)
static_tables = pinyin.tab swkb.dat symbols.dat

dist_noinst_DATA = \
	NOTE \
	phone.cin \
	phone.cin-CNS11643-complete.patch \
	svnrev \
	tsi.src \
	$(NULL)

chewing_datadir = $(pkgdatadir)
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
	env LC_ALL=C $(tooldir)/init_database$(EXEEXT) $(top_srcdir)/data/phone.cin $(top_srcdir)/data/tsi.src

CLEANFILES = $(datas) gendata_stamp
