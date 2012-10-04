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

all: checkdata_stamp gendata_stamp

checkdata_stamp: phone.cin tsi.src
	@echo "timestamp" > $@

gendata_stamp:
	-if test -f $(generated_header); then \
		echo "chewing-definition.h exists."; \
	else \
		$(MAKE) gendata; \
	fi
	@echo "timestamp" > $@

gendata:
	$(tooldir)/sort_word$(EXEEXT) $(top_srcdir)/data/phone.cin
	$(tooldir)/sort_dic$(EXEEXT) $(top_srcdir)/data/tsi.src
	$(tooldir)/maketree$(EXEEXT)
	-rm -f phoneid.dic
	-mv -f chewing-definition.h $(top_builddir)/src/

installdirs:
	$(mkinstalldirs) $(DESTDIR)$(datadir)/chewing

install-static-tables: $(static_tables) installdirs
	for file in $(static_tables); do \
		$(INSTALL_DATA) $(top_srcdir)/data/$$file $(DESTDIR)$(datadir)/chewing; \
	done

install-data-local: $(datas) install-static-tables installdirs
	$(INSTALL_DATA) $(datas) $(DESTDIR)$(datadir)/chewing

CLEANFILES = $(datas) gendata_stamp checkdata_stamp $(generated_header)
