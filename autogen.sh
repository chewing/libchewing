#! /bin/sh

if [ -d autom4te.cache ]; then
	rm -rf autom4te.cache
fi

set -x

if [ "x${ACLOCAL_DIR}" != "x" ]; then
	ACLOCAL_ARG=-I ${ACLOCAL_DIR}
fi

${ACLOCAL:-aclocal} ${ACLOCAL_ARG}
${LIBTOOLIZE:-libtoolize} -c --automake 
# intltoolize -c --automake
${AUTOMAKE:-automake} --add-missing --copy --include-deps
${AUTOCONF:-autoconf}
