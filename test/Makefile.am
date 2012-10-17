valgrind-check: testchewing test.txt
	@echo "Please waiting for the valgrind results..."
	libtool --mode=execute valgrind \
		--leak-check=full \
		--show-reachable=yes \
		testchewing < test.txt

test.txt: default-test.txt
	@if [ ! -f test.txt ]; then cp $(srcdir)/default-test.txt test.txt ; fi

dist_noinst_DATA = \
	default-test.txt \
	$(NULL)

TESTS = $(NATIVE_TESTS)
NATIVE_TESTS = \
	test-key2pho \
	test-utf8 \
	test-mmap \
	$(NULL)

check_HEADERS = \
	test_harness.h \
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
genkeystroke_LDADD = $(LDADD) $(NCURSESW_LIBS)
else
TEXT_UI_BIN=
endif

INCLUDES = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	-DCHEWING_DATA_PREFIX="\"$(top_srcdir)/data\"" \
	-DTEST_HASH_DIR="\"$(PWD)\"" \
	$(CHECK_CFLAGS) \
	$(NULL)

LDADD = \
	$(top_builddir)/src/libchewing.la \
	$(top_builddir)/src/common/libcommon.la \
	$(top_builddir)/src/porting_layer/src/libporting_layer.la \
	$(NULL)

CLEANFILES = uhash.dat materials.txt-random test.txt
