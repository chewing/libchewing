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
	autotools/config.guess \
	autotools/config.sub \
	autotools/depcomp \
	autotools/install-sh \
	autotools/ltmain.sh \
	autotools/missing \
	include/config.h.in
