#!/bin/bash

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

function gendiff()
{
    diff -u ${1}-${2} ${1}-${3} --label ${1} --label ${1} > ${1}.diff
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
if [ `echo $new_svnrev | cut -c2-` -gt `echo $old_svnrev | cut -c2-` ]; then
    # gendiff should only work in this case.
    echo
else
	echo "Given revision is ${new_svnrev} <= Original(${old_svnrev})."
    echo "No need to gendiff.  Abort!"
    exit 1
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
gendiff phone.cin $old_svnrev $new_svnrev
gendiff tsi.src   $old_svnrev $new_svnrev

echo "Done! Please check file phone.cin.diff and tsi.src.diff"

rm -f \
    phone.cin-$old_svnrev phone.cin-$new_svnrev \
    tsi.src-$old_svnrev tsi.src-$new_svnrev
