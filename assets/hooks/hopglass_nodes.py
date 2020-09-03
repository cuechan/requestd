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

hopglass_nodes = {
	'timestamp': datetime.datetime.utcnow().isoformat(),
	'version': 2,
	'nodes': []
}

hopglass_graph = {
	'timestamp': datetime.datetime.utcnow().isoformat(),
	'version': 1,
	'batadv': {
		'multigraph': False,
		'directed': False,
		'nodes': [],
		'links': []
	}
}

graph_nodes = []


# print(graph_nodes)


def get_graphnode_idx_by_iface(iface):
	for idx,node in enumerate(graph_nodes):
		if iface in node.get('interfaces', []):
			# found
			return idx

	return -1



for node in data:
	nodeinfo = node['last_response']['nodeinfo']
	hg_node = dict()
	links = []

	try:
		hg_node['flags'] = {'online': node['status'] == 'Up'}
		# print(node['status'])

		hg_node["nodeinfo"]   = node['last_response']['nodeinfo']
		hg_node['statistics'] = node['last_response']['statistics']
		hg_node['statistics'] = node['last_response']['statistics']
		hg_node['firstseen']  = node['first_seen']
		hg_node['lastseen']   = node['last_seen']


		# for ifacetype, mac in nodeinfo['network']['mesh']['bat0']['interfaces'].items():
		# 	# check if we mesh on this interface
		# 	# if mac in node['last_response']['neighbors']['batadv']:
		# 	pass


		# check batadv neighbours
		for iface,neighbours in  node['last_response']['neighbours']['batadv'].items():
			# do we have neighbors?
			# print(f"{iface}")
			if get_graphnode_idx_by_iface(iface) == -1:
				interfaces = [nodeinfo['network']['mac']]
				for x,a in nodeinfo['network']['mesh']['bat0']['interfaces'].items():
					interfaces.extend(a)

				graph_nodes.append({
					'node_id': node['nodeid'],
					'id': nodeinfo['network']['mac'],
					'interfaces': interfaces
				})

			myIndex = get_graphnode_idx_by_iface(nodeinfo['network']['mac'])

			for neighbour,vals in neighbours['neighbours'].items():
				# check if our neighbour is already in index. if so add the link
				if get_graphnode_idx_by_iface(neighbour) != -1:
					links.append({
						'source': myIndex,
						'target': get_graphnode_idx_by_iface(neighbour),
						'tq': 1 if vals['tq'] == 0 else 255 / vals['tq'],
						'type': 'batadv' # TODO: check interface type
					})

	except KeyError as e:
		print(f"{node['nodeid']} has incomplete response. missing {e}. ignore")
		# exit(0)

	hopglass_nodes['nodes'].append(hg_node)
	hopglass_graph['batadv']['links'].extend(links)


hopglass_graph['batadv']['nodes'] = [{'id': n['id'], 'node_id': n['node_id']} for n in graph_nodes]

json.dump(hopglass_nodes, sys.stdout)
# print(json.dumps(node, indent=4, sort_keys=True))
