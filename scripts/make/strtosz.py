#!/usr/bin/env python3

import argparse

parser = argparse.ArgumentParser()
parser.add_argument("size", type=str, help="Input string representing size")
args = parser.parse_args()

size = args.size.strip().lower()

if size.startswith("0x"):
    if size[-1] != "b":
        raise ValueError("Hexadecimal size must end with 'b' or 'B'")
    number = int(size[:-1], 16)
    multiplier = 1
else:
    # If last character is a digit, append 'm' (megabytes)
    if size[-1].isdigit():
        size += "m"

    suffixes = {
        "b": 0,
        "k": 1,
        "m": 2,
        "g": 3,
        "t": 4,
        "p": 5,
        "e": 6,
    }
    if size[-1] not in suffixes:
        raise ValueError("Invalid size suffix. Use one of b, k, m, g, t, p, e")
    multiplier = 1024 ** suffixes[size[-1]]
    number = float(size[:-1])

print(int(number * multiplier))
