#!/bin/sh

# Subversion repository of libchewing-data
DATA_BASE_URL=http://svn.openfoundry.org/libchewingdata/utf-8

function svn_checkout()
{
    svn cat -r $2 $DATA_BASE_URL/$1 > ${1}-${2}
}

function sort_uniq()
{
    TMP_FILE=/tmp/libchewing-data
    dos2unix $1 2>/dev/null
    sort $1 | uniq > /$TMP_FILE
    mv -f $TMP_FILE $1
}

function svnrev()
{
    if [ -f svnrev ]; then
        echo "svnrev"
    else
        if [ -f data/svnrev ]; then
            echo "data/svnrev"
        else
            echo "Error: file 'svnrev' not found!"
            exit 1
        fi
    fi
}

old_svnrev=`head -n1 $(svnrev)`
echo "Original SVN rev: $old_svnrev"

if [ x$1 = x ]; then
    new_svnrev=`svn info $DATA_BASE_URL | awk  '/Revision:/ {print $2}'`
    new_svnrev="r${new_svnrev}"
else
    new_svnrev=$1
fi

# Download original revision
svn_checkout phone.cin $old_svnrev
svn_checkout tsi.src $old_svnrev

echo "Expected SVN rev: $new_svnrev"

# Download expected revision
svn_checkout phone.cin $new_svnrev
svn_checkout tsi.src $new_svnrev

echo "Generating diff..."

# sort + uniq
sort_uniq phone.cin-$old_svnrev
sort_uniq phone.cin-$new_svnrev
sort_uniq tsi.src-$old_svnrev
sort_uniq tsi.src-$new_svnrev

# Generate diff
diff -u phone.cin-$old_svnrev phone.cin-$new_svnrev > phone.cin.diff
diff -u tsi.src-$old_svnrev tsi.src-$new_svnrev > tsi.src.diff

echo "Done! Please check file phone.cin.diff and tsi.src.diff"

rm -f \
    phone.cin-$old_svnrev phone.cin-$new_svnrev \
    tsi.src-$old_svnrev tsi.src-$new_svnrev
