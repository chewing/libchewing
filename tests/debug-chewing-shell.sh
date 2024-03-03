#!/bin/sh

PREFERABLE_XTERM=urxvt
MESSAGES="Please type 'run' inside gdb session to invoke genkeystroke.

After execution, close this X terminal."

$PREFERABLE_XTERM -e sh \
  -c "tty > /tmp/name_of_xterm_ptty && 
      echo -e \"$MESSAGES\" && sleep 100000" &
sleep 1
echo "tty `cat /tmp/name_of_xterm_ptty`" > .chewing-gdb-macro
echo "set args test.txt" >> .chewing-gdb-macro
libtool --mode=execute gdb -x .chewing-gdb-macro genkeystroke
rm -f .chewing-gdb-macro
