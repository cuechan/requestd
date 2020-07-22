#!/usr/bin/python3

import json
import sys
import subprocess
import os
from os.path import isfile, join
import re
import shutil


fastd_keyrepo = "git@ffhl-srv01:fastd-keys"
tmp_dir = "/tmp/fastd_autokeyimport"

node = json.load(sys.stdin)


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

def load_keys_from_repo():
	# get the key repo
	exitcode = subprocess.run(f"git clone {fastd_keyrepo} {tmp_dir}", shell=True)

	if exitcode != 0:
		eprint("can't get key-repo. maybe its already there?")

	keys = []

	for keyfile in [join(tmp_dir, f) for f in os.listdir(tmp_dir) if isfile(join(tmp_dir, f))]:
		# eprint(f"loading {keyfile}")

		f = open(keyfile, 'r')
		for line in f:
			m = re.match('^key "([a-fA-F0-9]{64})";', line)
			if m is None:
				continue
			keyfound = m.group(1)
			keys.append(keyfound)

	return keys




# check, if we got a key
# name = os.environ['NODE_name']
# print(f"Response from {name}")
# # time.sleep(0.1)


keys = load_keys_from_repo()

# print(json.dumps(node, indent=4, sort_keys=True))
# exit(0)


try:
	nodeid    = node['nodeinfo']['node_id']
	fastd_key = node['nodeinfo']['software']['fastd']['public_key']
	nodename  = node['nodeinfo']['hostname']
	contact   = node['nodeinfo']['owner']['contact']
except KeyError as e:
	eprint(f"{nodeid} sent incomplete response. ignoring. missing {e}")
	exit(0)

# check if key is in loaded keys
if not fastd_key in keys:
	print(f"{nodename} is not known and not registered")
	# now do something useful


shutil.rmtree(tmp_dir)
