#!/usr/bin/env python
import os
import subprocess
import re
import random

def show_command_to_reproduce(keystrokes):
    print '''echo '%s' | ./testchewing''' % keystrokes.strip()

def genkey(seed, length=100):
    p = subprocess.Popen(
            ['./randkeystroke', '-r', '-s', str(seed), '-n', str(length)],
            stdout=subprocess.PIPE)
    return p.communicate()[0]

def reset_state():
    if os.path.exists('uhash.dat'):
        os.remove('uhash.dat')

def run(keystrokes):
    reset_state()

    p = subprocess.Popen(
            './testchewing',
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE)
    p.communicate(keystrokes)
    return p.returncode

def parse_keys(keystrokes):
    return re.findall(r'<[^<>]+>|[^<>]', keystrokes)

def try_to_reduce_key(keystrokes):
    keys = parse_keys(keystrokes)

    while True:
        for i in range(len(keys)):
            keys2 = keys[0:i] + keys[i+1:]
            keystrokes = ''.join(keys2)
            if run(keystrokes) != 0:
                keys = keys2
                #print len(keys2)
                #show_command_to_reproduce(keystrokes)
                break
        else:
            show_command_to_reproduce(keystrokes)
            return

def main():
    for seed in xrange(10**9):
        # show progress
        if seed % 1000 == 0:
            print 'seed', seed

        keystroke = genkey(seed)
        if run(keystroke) != 0:
            print 'failed for seed', seed
            show_command_to_reproduce(keystroke)
            try_to_reduce_key(keystroke)
            #break



if __name__ == '__main__':
    main()
