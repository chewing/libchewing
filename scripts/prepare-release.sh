#!/bin/sh

CHEWING=libchewing
VERSION=`cat configure.ac | awk  '/AC_INIT/ {print $2}' | tr -d ' ' | sed -e 's/,//'`
NEW=$CHEWING-$VERSION
TARBALL=$NEW.tar.gz
SVNENTRY=.svn/entries

if [ ! -f $SVNENTRY ]; then
	echo "Error! This script is only for developers."
	exit
fi

rm -rf trunk $NEW $TARBALL

svn co `grep "url=" $SVNENTRY | cut -d\" -f2`

if [ ! -d trunk ]; then
	echo "Error!"
	exit
fi

mv trunk $NEW

pushd $NEW
# Use the fresh autotool.
sh autogen.sh

find -name .svn | xargs rm -rf
find -name .cvsignore | xargs rm -f

# Remove local Debian information to make sure upstream be clean.
rm -rf debian

# Remove automake's cache
rm -rf autom4te.cache
popd

tar zcvf $TARBALL $NEW
rm -rf $NEW

# Show some info
echo -e "\033[44;37m Congratulation. Newer release was generated. Please check out its tarball and MD5. \033[m"
md5sum $TARBALL
ls -lh $TARBALL
