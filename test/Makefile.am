valgrind-check: testchewing test.txt
	@echo "Please waiting for the valgrind results..."
	libtool --mode=execute valgrind \
		--leak-check=full \
		--show-reachable=yes \
		./testchewing < test.txt

test.txt: default-test.txt
	@if [ ! -f test.txt ]; then cp $(srcdir)/default-test.txt test.txt ; fi

dist_noinst_DATA = \
	default-test.txt \
	$(NULL)

noinst_LTLIBRARIES = libtesthelper.la

libtesthelper_la_SOURCES = \
	testhelper.c \
	$(NULL)

libtesthelper_la_LIBADD = \
	$(top_builddir)/src/libchewing.la \
	$(NULL)

TESTS = $(NATIVE_TESTS)
NATIVE_TESTS = \
	test-bopomofo \
	test-config \
	test-easy-symbol \
	test-fullshape \
	test-key2pho \
	test-keyboard \
	test-logger \
	test-mmap \
	test-path \
	test-reset \
	test-regression \
	test-symbol \
	test-special-symbol \
	test-userphrase \
	test-utf8 \
	$(NULL)

check_HEADERS = \
	testhelper.h \
	$(NULL)

check_PROGRAMS = \
	testchewing \
	simulate \
	randkeystroke \
	$(TEXT_UI_BIN) \
	$(NATIVE_TESTS) \
	$(NULL)

test_mmap_CPPFLAGS = -DTESTDATA="\"$(srcdir)/default-test.txt\""

if ENABLE_TEXT_UI
TEXT_UI_BIN=genkeystroke
genkeystroke_SOURCES = gen_keystroke.c
genkeystroke_LDADD = $(LDADD) @CURSES_LIB@
else
TEXT_UI_BIN=
endif

INCLUDES = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	-I$(top_srcdir)/test \
	-DCHEWING_DATA_PREFIX="\"$(top_builddir)/data\"" \
	-DTEST_HASH_DIR="\"$(PWD)\"" \
	$(CHECK_CFLAGS) \
	$(NULL)

LDADD = \
	$(top_builddir)/src/common/libcommon.la \
	$(top_builddir)/src/porting_layer/src/libporting_layer.la \
	$(top_builddir)/test/libtesthelper.la \
	$(NULL)

AM_LDFLAGS = -static

CLEANFILES = uhash.dat materials.txt-random test.txt
