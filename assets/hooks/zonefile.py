#!/usr/bin/python3
import json
import sys
import socket
import os
import re
from ipaddress import *
from time import time

# load environment vars
CONTROLSOCKET = os.environ.get('REQUESTD_CTRLSOCKET', '/var/run/requestd.sock')
OUTFILE = os.environ.get('OUT', '/tmp/nodes.zone')
SOA = os.environ.get('SOA', 'srv02.luebeck.freifunk.net.')
NS = os.environ.get('NS', 'srv02.luebeck.freifunk.net.')

ValidHostnameRegex = "^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$"
#prefix = IPv6Network('fdef:ffc0:3dd7::/64')
prefix = IPv6Network('2001:67c:2d50::/48')

def get_all_nodes():
	s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
	s.connect(CONTROLSOCKET)
	data = bytearray()
	# maybe not the perfect way but it works for now
	buffer = bytearray(1)
	while s.recv_into(buffer,1):
		data += buffer

	return json.loads(data)


# TODO: save data to a file instead of stdout

data = get_all_nodes()

f = open(OUTFILE, "wt")



f.write(f"""$TTL 600  ; 10 minutes
@     IN SOA  {SOA} info.luebeck.freifunk.net. (
					{int(time())} ; serial
					600        ; refresh (10min)
					30         ; retry (30s)
					3600       ; expire (1 hour)
					60         ; minimum (1 minute)
					)
		NS {NS}

""")

HostnameRegex = re.compile(ValidHostnameRegex)

for e in data:
	node = e["last_response"]["nodeinfo"]
	try:
		hostname = node['hostname']
		if HostnameRegex.match(hostname) == None:
			continue

		address = None

		for a in node['network']['addresses']:
			a = IPv6Address(a)
			if a in prefix:
				address = a
				break

		if address:
			f.write(f"{hostname}\tAAAA\t{address}\n")
	except:
		pass
