tooldir = $(top_builddir)/src/tools
datas = us_freq.dat ch_index.dat dict.dat ph_index.dat fonetree.dat pinyin.tab
all: gendata
gendata: phone.cin tsi.src
	$(tooldir)/sort_word
	$(tooldir)/sort_dic tsi.src
	$(tooldir)/maketree
	-rm -f phoneid.dic
install-data-local: $(datas) pinyin.tab
	$(mkinstalldirs) $(DESTDIR)/$(datadir)/chewing
	$(INSTALL_DATA) $(datas) $(DESTDIR)/$(datadir)/chewing
CLEANFILES = $(datas)
