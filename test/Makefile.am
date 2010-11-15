testdir = $(top_builddir)/test
TESTDATA = $(testdir)/test.txt
DEFAULT_TESTDATA = $(top_srcdir)/test/default-test.txt
LIBCHEWING = \
	$(top_builddir)/src/libchewing.la \
	$(top_builddir)/src/common/libcommon.la \
	$(top_builddir)/src/porting_layer/src/libporting_layer.la
DATADIR = $(top_builddir)/data
if ENABLE_BINARY_DATA
CHINDEXS = $(DATADIR)/ch_index_begin.dat \
		   $(DATADIR)/ch_index_phone.dat
else
CHINDEXS = $(DATADIR)/ch_index.dat
endif
CHEWINGDATA = \
	$(DATADIR)/us_freq.dat \
	$(DATADIR)/dict.dat \
	$(DATADIR)/ph_index.dat \
	$(DATADIR)/fonetree.dat \
	$(DATADIR)/swkb.dat \
	$(DATADIR)/symbols.dat \
	$(CHINDEXS)

$(TESTDATA): $(DEFAULT_TESTDATA)
	@if [ ! -f $(TESTDATA) ]; then cp $(DEFAULT_TESTDATA) $(TESTDATA) ; fi

check: testchewing $(TESTDATA) $(CHEWINGDATA)
	$(testdir)/testchewing < $(TESTDATA)
if ENABLE_UNIT_TEST
	$(testdir)/do-test
endif

vcheck: check
	@echo "Please waiting for the valgrind results..."
	libtool --mode=execute valgrind \
		--leak-check=full \
		--show-reachable=yes \
		$(testdir)/testchewing < $(TESTDATA)

bin_PROGRAMS = \
	testchewing \
	simulate \
	randkeystroke \
	$(TEXT_UI_BIN) \
	$(UNIT_TEST_BIN)

INCLUDES = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-DCHEWING_DATA_PREFIX="\"$(DATADIR)\"" \
	-DTEST_HASH_DIR="\"$(PWD)\"" \
	$(CHECK_CFLAGS) \
	$(NCURSESW_CFLAGS) \
	-I$(top_srcdir)/src/porting_layer/include

testchewing_SOURCES = testchewing.c
testchewing_LDADD = $(LIBCHEWING)
testchewing_DEPENDENCIES = $(LIBCHEWING)

simulate_SOURCES = simulate.c
simulate_LDADD = $(LIBCHEWING)
simulate_DEPENDENCIES = $(LIBCHEWING)

if ENABLE_TEXT_UI
TEXT_UI_BIN=genkeystroke
genkeystroke_SOURCES = gen_keystroke.c
genkeystroke_LDADD = $(LIBCHEWING) $(NCURSESW_LIBS)
genkeystroke_DEPENDENCIES = $(LIBCHEWING)
else
TEXT_UI_BIN=
endif

if ENABLE_UNIT_TEST
UNIT_TEST_BIN=do-test
test_case_SRC = \
	test-utf8.c \
	test-key2pho.c \
	test-mmap.c
do_test_SOURCES = \
	do-test.c \
	$(test_case_SRC)
do_test_LDADD = \
	$(LIBCHEWING) \
	$(CHECK_LIBS)
do_test_DEPENDENCIES = $(LIBCHEWING)
else
UNIT_TEST_BIN=
endif

CLEANFILES = uhash.dat materials.txt-random
