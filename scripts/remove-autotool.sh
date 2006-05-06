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
	config.sub \
	config.guess \
	configure \
	install-sh \
	config.log \
	depcomp \
	libtool \
	config.status \
	ltmain.sh \
	missing \
	mkinstalldirs
