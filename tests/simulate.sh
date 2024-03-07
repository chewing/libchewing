#!/bin/sh

abort() {
    echo $@
    exit 1
}

do_simulate() {
    ./simulate
}

do_rand_simulate() {
     sort -R materials.txt | egrep -v "^#" > materials.txt-random
     do_simulate
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
if [ x"$1" = "xrandom" ]; then
    rm -f uhash.dat
    do_rand_simulate
else
    do_simulate
fi
