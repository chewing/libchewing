tooldir = $(top_builddir)/src/tools
datas = \
	us_freq.dat \
	ch_index.dat \
	dict.dat \
	ph_index.dat \
	fonetree.dat
all: gendata convert_dat gendata2

gendata: phone.cin tsi.src
	$(tooldir)/sort_word
	$(tooldir)/sort_dic $(top_srcdir)/data/tsi.src
	$(tooldir)/maketree
	-rm -f phoneid.dic
	-mv -f chewing-definition.h ../src
if ENABLE_BINARY_DATA
convert_dat:
	$(tooldir)/dat2bin
gendata2: ch_index.dat_bin fonetree.dat_bin ph_index.dat_bin
	-mv -f ch_index.dat_bin ch_index.dat
	-mv -f fonetree.dat_bin fonetree.dat
	-mv -f ph_index.dat_bin ph_index.dat
else
convert_dat:
	@echo "Nothing in convert_dat"
gendata2:
	@echo "Nothing in gendata2"
endif

install-data-local: $(datas) pinyin.tab
	$(mkinstalldirs) $(DESTDIR)/$(datadir)/chewing
	$(INSTALL_DATA) $(datas) $(DESTDIR)/$(datadir)/chewing
CLEANFILES = $(datas)
