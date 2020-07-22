#!/usr/bin/python3

import json
import sys
import time
import os


node = json.load(sys.stdin)
# print(json.dumps(node, indent=4, sort_keys=True))
name = os.environ['NODE_name']
# print(f"Response from {name}")
