valgrind-check: testchewing
	@echo "Please waiting for the valgrind results..."
	libtool --mode=execute valgrind \
		--leak-check=full \
		--show-reachable=yes \
		./testchewing < $(srcdir)/data/default-test.txt

noinst_LTLIBRARIES = libtesthelper.la

dist_noinst_DATA = \
	data/default-test.txt \
	data/dictionary.dat \
	data/index_tree.dat \
	data/phone.cin \
	data/pinyin.tab \
	data/swkb.dat \
	data/symbols.dat \
	data/tsi.src \
	$(NULL)

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
	test-error-handling \
	test-fullshape \
	test-key2pho \
	test-keyboard \
	test-keyboardless \
	test-logger \
	test-mmap \
	test-path \
	test-reset \
	test-regression \
	test-symbol \
	test-special-symbol \
	test-struct-size \
	test-userphrase \
	test-utf8 \
	$(NULL)

check_HEADERS = \
	testhelper.h \
	$(NULL)

check_PROGRAMS = \
	performance \
	testchewing \
	simulate \
	randkeystroke \
	$(TEXT_UI_BIN) \
	$(NATIVE_TESTS) \
	$(NULL)

if ENABLE_TEXT_UI
TEXT_UI_BIN=genkeystroke
genkeystroke_SOURCES = genkeystroke.c
genkeystroke_LDADD = $(LDADD) @CURSES_LIB@
else
TEXT_UI_BIN=
endif

AM_CPPFLAGS = \
	-I$(top_srcdir)/include \
	-I$(top_srcdir)/include/internal \
	-I$(top_srcdir)/src/porting_layer/include \
	-I$(top_srcdir)/test/ \
	-I$(top_srcdir)/thirdparty/sqlite-amalgamation \
	-DCHEWING_DATA_PREFIX="\"$(top_builddir)/data\"" \
	-DTEST_HASH_DIR="\"$(PWD)\"" \
	-DTEST_DATA_DIR="\"$(srcdir)/data\"" \
	$(DEFAULT_CPPFLAGS) \
	$(CHECK_CFLAGS) \
	$(NULL)

LDADD = \
	$(top_builddir)/src/common/libcommon.la \
	$(top_builddir)/src/porting_layer/src/libporting_layer.la \
	$(top_builddir)/test/libtesthelper.la \
	$(NULL)

AM_LDFLAGS = -static

if WITH_SQLITE3
if WITH_INTERNAL_SQLITE3
sqlite3_SOURCES = \
	$(top_srcdir)/thirdparty/sqlite-amalgamation/shell.c \
	$(top_srcdir)/thirdparty/sqlite-amalgamation/sqlite3.c \
	$(NULL)

check_PROGRAMS += \
	sqlite3 \
	$(NULL)
endif
endif

CLEANFILES = uhash.dat materials.txt-random chewing.sqlite3 test.sqlite3
