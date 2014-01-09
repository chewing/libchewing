#!/usr/bin/env python3
import argparse
import os
import sys


def get_args():
    parser = argparse.ArgumentParser(
        description='Create keystroke from text.')
    parser.add_argument('input', metavar='input', type=str, nargs=1,
        help='Input Chinese text file')
    parser.add_argument('output', metavar='output', type=str, nargs=1,
        help='Output keystroke file')
    parser.add_argument('phone', metavar='phone', type=str, nargs='?',
        default=os.path.join(os.path.dirname(sys.argv[0]), '..', 'data', 'phone.cin'),
        help='phone.cin')
    return parser.parse_args()


def read_phone(phone):
    phone_table = {}
    with open(phone) as f:
        for l in f:
            if l.startswith('%chardef  begin'):
                break
        for l in f:
            if l.startswith('%chardef  end'):
                break
            item = l.split()
            phone_table[item[1]] = item[0]
    return phone_table


def main():
    args = get_args()
    phone_table = read_phone(args.phone)

    in_file = args.input[0]
    out_file = args.output[0]
    word_count = 0

    with open(in_file) as in_, open(out_file, "w") as out_:
        for l in in_:
            for c in l:
                if c in phone_table:
                    out_.write(phone_table[c])
                    word_count += 1
        out_.write('\n')
    print('word count = {}'.format(word_count))


if __name__ == '__main__':
    main()
