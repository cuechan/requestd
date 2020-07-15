#!/usr/bin/python3

import json
import sys

# read the data from stdin
node = json.load(sys.stdin)

print(f"Node is offline: {node['nodeinfo']['hostname']} ")
