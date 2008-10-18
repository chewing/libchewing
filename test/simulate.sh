#!/bin/sh

abort() {
    echo $@
    exit 1
}

if [ ! -f materials.txt ]; then
    abort "Error! file materials.txt doesn't exist."
fi
if [ ! -f testchewing ]; then
    echo "program 'testchewing' not found.  Try to make"
    make >/dev/null
    if test "$?" != "0"; then
        abort "Error! make fails"
    fi
fi

clear
echo "Simulating..."
echo "---"
cat materials.txt | sed 's/^#.*//' | while read line ; do
if test "x$line" != "x"; then
    echo -n "[Committed] "
    echo $line | awk '{split($0,x,"<E>"); print x[1]"<E>"}' | ./testchewing

    echo -n "[Expected]  "
    echo $line | awk '{split($0,x,"<E>"); print x[2]}' | sed 's/^ //g'
    echo "---"
fi
done

