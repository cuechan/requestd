#!/usr/bin/python3

import json
import sys


node = json.load(sys.stdin)
# print(json.dumps(node, indent=4, sort_keys=True))


print(f"Node came back: {node['nodeinfo']['hostname']}")
