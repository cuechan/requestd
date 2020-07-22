#!/usr/bin/python3

import json
import sys


# load json from stdin
node = json.load(sys.stdin)

print(json.dumps(node, indent=4, sort_keys=True))
print(f"script was triggered for {node['nodeinfo']['hostname']}")
