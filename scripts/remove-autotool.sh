#!/bin/sh

if [ -f Makefile ]; then
	if [ -f test/Makefile ]; then
		make -C test distclean
	fi
	make maintainer-clean
fi

find -name .deps | xargs rm -rf

rm -f \
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
