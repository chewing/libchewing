#! /bin/sh

AUTORECONF_ARGS=-i
mkdir -p m4
AUTORECONF_ARGS="$AUTORECONF_ARGS -I m4"

gnulib-tool --libtool --import \
        strtok_r \
        ;
autoreconf $AUTORECONF_ARGS
