#!/bin/sh

abort() {
    echo $@
    exit 1
}

if [ ! -f materials.txt ]; then
    abort "Error! file materials.txt doesn't exist."
fi
if [ ! -f simulate ]; then
    echo "program 'simulate' not found.  Try to make"
    make >/dev/null 2>/dev/null
    if test "$?" != "0"; then
        abort "Error! Fail to build simulate"
    fi
fi

clear
# Simply invoke simulate
./simulate
