#!/usr/bin/env python3

# This uses `ucd-generate` to generate all of the internal tables.

import argparse
import os
import subprocess
import sys


def eprint(*args, **kwargs):
    kwargs['file'] = sys.stderr
    print(*args, **kwargs)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('ucd', metavar='DIR', nargs=1)
    args = p.parse_args()
    ucd = args.ucd[0]

    def generate(subcmd, *args, **kwargs):
        subprocess.run(("ucd-generate", subcmd, ucd) + args, check=True, **kwargs)

    def generate_file(path, subcmd, *args, filename=None, **kwargs):
        if filename is None:
            filename = subcmd.replace('-', '_') + ".rs"
        eprint('-', filename)
        with open(os.path.join(path, filename), "w") as f:
            generate(subcmd, *args, stdout=f, **kwargs)

    def generate_fst(path, subcmd, *args, **kwargs):
        eprint('-', subcmd)
        generate(subcmd, *args, "--fst-dir", path, **kwargs)

    eprint('generating ucd-trie tables')
    path = os.path.join("ucd-trie", "src")
    generate_file(path, "general-category")

    eprint('generating ucd-util tables')
    path = os.path.join("ucd-util", "src", "unicode_tables")
    generate_file(path, "jamo-short-name")
    generate_file(path, "property-names")
    generate_file(path, "property-values")

    eprint('generating benches/tables/fst')
    path = os.path.join("benches", "tables", "fst")
    generate_fst(path, "general-category", "--exclude", "unassigned", "--enum")
    generate_fst(path, "jamo-short-name")
    generate_fst(path, "names", "--no-aliases", "--no-hangul", "--no-ideograph")

    eprint('generating benches/tables/slice')
    path = os.path.join("benches", "tables", "slice")
    generate_file(path, "general-category", "--exclude", "unassigned",
                  filename="general_categories.rs")
    generate_file(path, "general-category", "--exclude", "unassigned", "--enum")
    generate_file(path, "jamo-short-name")
    generate_file(path, "names", "--no-aliases", "--no-hangul", "--no-ideograph")

    eprint('generating benches/tables/trie')
    path = os.path.join("benches", "tables", "trie")
    generate_file(path, "general-category", "--exclude", "unassigned", "--trie-set",
                  filename="general_categories.rs")

if __name__ == '__main__':
    main()
