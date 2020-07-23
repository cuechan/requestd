#!/usr/bin/python3

import json
import sys
from syslog import syslog

# ideas:
# - send a welcome mail if contact info is a mail address

node = json.load(sys.stdin)
# print(json.dumps(node, indent=4, sort_keys=True))


print(f"New Node: {node['nodeinfo']['hostname']}")
