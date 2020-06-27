#!/usr/bin/python3

import json
import sys

# do stuff an generate a bind file

node = json.load(sys.stdin)


print(f"A new Node!!! {node['nodeinfo']['node_id']}")
