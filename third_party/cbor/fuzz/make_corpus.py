#!/usr/bin/env python

import base64
import json
import sys
import os.path

out_dir = sys.argv[1]
os.makedirs(out_dir)

with open("appendix_a.json") as f:
    appendix = json.load(f)

for i, entry in enumerate(appendix):
    buf = base64.b64decode(entry["cbor"])
    with open(os.path.join(out_dir, str(i)), 'wb') as f:
        f.write(buf)
