#!/bin/sh

if [ -f Makefile ]; then
	if [ -f test/Makefile ]; then
		make -C test distclean
	fi
	make maintainer-clean
fi

find -name .deps | xargs rm -rf
find -name stamp-h1 | xargs rm -f

rm -rf autom4te.cache

rm -f \
	aclocal.m4 \
	configure \
	config.log \
	libtool \
	config.status \
	mkinstalldirs \
	build-aux/config.guess \
	build-aux/config.sub \
	build-aux/depcomp \
	build-aux/install-sh \
	build-aux/ltmain.sh \
	build-aux/missing \
	build-aux/texinfo.tex \
	build-aux/mdate-sh \
	include/config.h.in
