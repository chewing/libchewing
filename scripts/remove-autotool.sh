#!/bin/sh

if [ -f Makefile ]; then
	if [ -f test/Makefile ]; then
		make -C test distclean
	fi
	make distclean
fi

rm -rf autom4te.cache

find -name Makefile | xargs rm -f 
find -name Makefile.in | xargs rm -f
find -name .deps | xargs rm -rf

rm -rf \
	aclocal.m4 \
	config.sub \
	config.guess \
	configure \
	install-sh \
	autom4te.cache \
	config.log \
	depcomp \
	libtool \
	config.status \
	ltmain.sh \
	missing \
	mkinstalldirs
