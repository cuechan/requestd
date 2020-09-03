#!/usr/bin/python3
import json
import sys
import socket
import os
import re
from ipaddress import *
from time import time
import datetime

# load environment vars
CONTROLSOCKET = os.environ.get('REQUESTD_CTRLSOCKET', '/tmp/requestd.sock')


def eprint(*args, **kwargs):
	print(*args, file=sys.stderr, **kwargs)


def get_all_nodes():
	s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
	s.connect(CONTROLSOCKET)
	data = bytearray()
	# maybe not the perfect way but it works for now
	buffer = bytearray(1)
	while s.recv_into(buffer,1):
		data += buffer
	return json.loads(data)


data = get_all_nodes()
# print(json.dumps(data, indent=4, sort_keys=True))

hopglass_graph = {
	'timestamp': datetime.datetime.utcnow().isoformat(),
	'version': 1,
	'batadv': {
		'multigraph': False,
		'directed': True,
		'nodes': [],
		'links': []
	}
}

graph_nodes = []

# print(graph_nodes)


def get_graphnode_idx_by_mac(mac):
	for idx,node in enumerate(graph_nodes):
		if mac in node['ifaces']:
			return idx
	return -1


def get_graphnode_idx_by_id(mac):
	for idx,node in enumerate(graph_nodes):
		if node['id'] == mac:
			return idx
	return -1


for node in data:
	nodeinfo = node['last_response']['nodeinfo']
	nodeid = nodeinfo['node_id']
	id = nodeinfo['network']['mac']
	links = []

	if node['status'] != 'Up':
		continue

	graphnode = {
		'node_id': nodeid,
		'id': id,
		'ifaces': {}
	}

	try:
		for type_, addresses in nodeinfo['network']['mesh']['bat0']['interfaces'].items():
			if type_ == 'tunnel':
				type_ = 'fastd'
			for addr in addresses:
				graphnode['ifaces'][addr] = type_


		graph_nodes.append(graphnode)

	except KeyError as e:
		eprint(f"{node['nodeid']} has incomplete response. missing {e}. ignore")
		# exit(0)

# print(json.dumps(graph_nodes, indent=4))

for node in data:
	nodeinfo = node['last_response']['nodeinfo']
	nodeid = nodeinfo['node_id']
	links = []

	if node['status'] != 'Up':
		continue

	try:
		# check batadv neighbours
		for iface,neighbours in  node['last_response']['neighbours']['batadv'].items():
			myIndex = get_graphnode_idx_by_mac(iface)

			for remote_iface,vals in neighbours['neighbours'].items():
				# check if our neighbour is already in index. if so add the link
				# eprint(neighbour, vals)

				neighbour_idx = get_graphnode_idx_by_mac(remote_iface)
				if neighbour_idx < 0:
					continue

				links.append({
					'source': myIndex,
					'target': neighbour_idx,
					'tq': 1 if vals['tq'] == 0 else 255 / vals['tq'],
					'type': graph_nodes[myIndex]['ifaces'][iface]
				})

	except KeyError as e:
		eprint(f"{node['nodeid']} has incomplete response. missing {e}. ignore")
		# exit(0)

	hopglass_graph['batadv']['links'].extend(links)


hopglass_graph['batadv']['nodes'] = [{'id': n['id'], 'node_id': n['node_id']} for n in graph_nodes]


json.dump(hopglass_graph, sys.stdout, indent=4)
