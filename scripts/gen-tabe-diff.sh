#!/bin/sh

if [ ! -f scripts/gen-tabe-diff.sh ]; then
  echo -e "\033[44;37m You *MUST* run this script in top dir. \033[m"
  exit
fi

if [ ! -f data/tabe-tsi.src ]; then
  echo -e "\033[44;37m tabe-tsi.src not found. Try to generate. \033[m"
  sh ./scripts/cvsup-tsi-src.sh
fi

# compare 
sort data/tsi.src | cut --delimiter=' ' -f1 > /tmp/orig-tsi
sort data/tabe-tsi.src | cut --delimiter=' ' -f1 > /tmp/tabe-tsi
diff -u /tmp/orig-tsi /tmp/tabe-tsi > tabe-tsi.diff
rm -f /tmp/orig-tsi /tmp/tabe-tsi

echo -e "\033[44;37m Please check out tabe-tsi.diff \033[m"
