#! /bin/sh

AUTORECONF_ARGS=-i
if [ -d m4 ]; then
    AUTORECONF_ARGS="$AUTORECONF_ARGS -I m4"
fi

autoreconf $AUTORECONF_ARGS
