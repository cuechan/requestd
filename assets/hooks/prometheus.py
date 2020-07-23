#!/usr/bin/python3
import json
import sys
import socket
import os
import re
from ipaddress import *
from time import time
import datetime
from prometheus_client import CollectorRegistry, Gauge, Counter, push_to_gateway

# load environment vars
CONTROLSOCKET = os.environ.get('REQUESTD_CTRLSOCKET', '/var/run/requestd.sock')
OUTFILE = "./"
PUSHGATEWAY = '176.9.147.120:9091'


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


registry = CollectorRegistry(auto_describe=True)

default_labels = ['nodeid', 'hostname', 'fw']


online = Gauge('knoten_online', 'online', default_labels, namespace='gluon', registry=registry)
clients = Gauge('knoten_clients', 'clients', default_labels, namespace='gluon', registry=registry)
uptime = Gauge('knoten_uptime', 'uptime', default_labels, namespace='gluon', registry=registry)
traffic = Gauge('knoten_traffic', 'traffic', default_labels + ['type'], namespace='gluon', registry=registry)
loadavg = Gauge('knoten_loadavg', 'loadavg', default_labels, namespace='gluon', registry=registry)
rootfs = Gauge('knoten_rootfs', 'rootfs', default_labels, namespace='gluon', registry=registry)
time = Gauge('knoten_time', 'time', default_labels, namespace='gluon', registry=registry)
wireless = Gauge('knoten_wireless', 'wireless', default_labels+['type'], namespace='gluon', registry=registry)
process = Gauge('knoten_process', 'process', default_labels+['type'], namespace='gluon', registry=registry)
meshvpn = Gauge('knoten_meshvpn', 'meshvpn', default_labels+['peer', 'group'], namespace='gluon', registry=registry)
cpu = Gauge('knoten_cpu', 'cpu', default_labels+['mode'], namespace='gluon', registry=registry)

memory_usage = Gauge('knoten_memory', 'memory usage', default_labels, namespace='gluon', registry=registry)
memory_total = Gauge('knoten_memory_total', 'memory total', default_labels, namespace='gluon', registry=registry)
memory_free = Gauge('knoten_memory_free', 'memory free', default_labels, namespace='gluon', registry=registry)

nodes_total  = Gauge('knoten_total', 'total online nodes', namespace='gluon', registry=registry)
nodes_online = Gauge('total_online', 'total online nodes', namespace='gluon', registry=registry)
clients_total = Gauge('clients_total', 'clients total', namespace='gluon', registry=registry)
traffic_total = Gauge('traffic_total', 'traffic total', ['type'], namespace='gluon', registry=registry)




for node in data:
	nodes_total.inc()


	nid = node['nodeid']
	d = node['last_response']
	hostname = d['nodeinfo']['hostname']


	# default labels
	deflbl = {
		'nodeid': nid,
		'hostname': hostname,
		'fw': d['nodeinfo']['software']['firmware']['release']
	}

	# check node status
	if node['status'] != 'Up':
		online.labels(**deflbl).set(0)
		continue

	online.labels(**deflbl).set(1)
	nodes_online.inc()


	try:

		# clients. per node, then increase global count
		clients.labels(**deflbl).set(d['statistics']['clients']['total'])
		clients_total.inc(amount=d['statistics']['clients']['total'])

		uptime.labels(**deflbl).set(d['statistics']['uptime'])
		loadavg.labels(**deflbl).set(d['statistics']['loadavg'])

		# traffic for different types (managemnt, forward, etc)
		for t,v in d['statistics']['traffic'].items():
			traffic.labels(**deflbl, type=t).set(v['bytes']) # per node
			traffic_total.labels(type=t).inc(amount=v['bytes']) # total counter

		# get info about wireless ifaces. some devices dont have wifi. use get with default to prevent exception
		for dev in d['statistics'].get('wireless', []):
			for t,v in dev.items():
				wireless.labels(**deflbl, type=t).set(v) # per node

		# meshvpn connections
		for group,peers in d['statistics'].get('mesh_vpn', {}).get('groups', {}).items():
			for p,v in peers.get('peers',{}).items():
				if v is not None:
					meshvpn.labels(**deflbl, group=group, peer=p).set(v['established']) # per node


		# cpu stats
		for m,v in d['statistics']['stat']['cpu'].items():
			cpu.labels(**deflbl, mode=t).set(v) # per node

		# memory
		memory_usage.labels(**deflbl).set(1-(d['statistics']['memory']['free']/d['statistics']['memory']['total']))
		memory_free.labels(**deflbl).set(d['statistics']['memory']['free'])
		memory_total.labels(**deflbl).set(d['statistics']['memory']['total'])

		rootfs.labels(**deflbl).set(d['statistics']['rootfs_usage'])
		time.labels(**deflbl).set(d['statistics']['time'])

		process.labels(**deflbl, type='total').set(d['statistics']['processes']['total'])
		process.labels(**deflbl, type='running').set(d['statistics']['processes']['running'])




	except KeyError as e:
		print(f"{node['nodeid']} has incomplete response. missing {e}. ignore")
		# exit(0)


push_to_gateway(PUSHGATEWAY, job='knoten', registry=registry)

# with open(FILEGRAPH, 'w') as outfile:
# 	json.dump(hopglass_graph, outfile)
