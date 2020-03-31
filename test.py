#!/usr/bin/python3

import json
import sys


node = json.load(sys.stdin)

# do things with the json
print("script has run: ", node['neighbours']['batadv'])
