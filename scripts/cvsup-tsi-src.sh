#!/bin/sh

TOP=`pwd`
TABE_TSI=$TOP/data/tabe-tsi.src

if [ ! -f scripts/cvsup-tsi-src.sh ]; then
  echo -e "\033[44;37m You *MUST* run this script in top dir. \033[m"
  exit
fi
if [ -f $TABE_TSI ]; then
  echo -e "\033[44;37m tabe-tsi.src done. \033[m"
  exit
fi

if [ ! -d libtabe ]; then
  echo -e "\033[44;37m No libtabe found. cvs check from libtabe \033[m" 
  cvs -z9 -d :pserver:xcin@xcin.linux.org.tw:/home/service/cvsroot/xcin \
	checkout libtabe
fi

if [ ! -d libtabe ]; then
  echo -e "\033[44;37m There were some problems while checking out. \033[m"
  exit
fi

pushd libtabe
./configure
if [ -f Makefile ]; then
  make clean all

  # patch tsidump to adapt Chewing
  pushd util
  if [ ! -f PATCHED ]; then
    patch -p0 < $TOP/scripts/tsi-chewing.diff
    touch PATCHED
  fi
  make
  
  # generation
  if [ -f tsidump ]; then
    ./tsidump -d ../tsi-src/tsi.db > $TABE_TSI
  fi
  popd
fi
popd

rm -rf libtabe

