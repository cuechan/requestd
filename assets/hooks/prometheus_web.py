#!/usr/bin/python3
import json
import sys
import socket
import os
import re
from ipaddress import *
from time import time
import datetime
from prometheus_client import CollectorRegistry, Gauge, Counter, push_to_gateway, delete_from_gateway
import prometheus_client

# load environment vars
CONTROLSOCKET = os.environ.get('REQUESTD_CTRLSOCKET', '/tmp/requestd.sock')
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
# data = json.load(sys.stdin)
# print(json.dumps(data, indent=4, sort_keys=True))





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
meshvpn_contime = Gauge('knoten_meshvpn', 'conected fastd instance', default_labels+['group', 'peer'], namespace='gluon', registry=registry)
cpu = Gauge('knoten_cpu', 'cpu', default_labels+['mode'], namespace='gluon', registry=registry)

memory_usage = Gauge('knoten_memory_usage', 'memory usage', default_labels, namespace='gluon', registry=registry)
memory_total = Gauge('knoten_memory_total', 'memory total', default_labels, namespace='gluon', registry=registry)
memory = Gauge('knoten_memory', 'memory', default_labels+['type'], namespace='gluon', registry=registry)


batman_adv = Gauge('knoten_batadv_compat', 'batman compat', default_labels+['compat'], namespace='gluon', registry=registry)
domain_counter = Gauge('domain_total', 'domain code', default_labels+['domain'], namespace='gluon', registry=registry)
nodes_total  = Gauge('knoten_total', 'total online nodes', namespace='gluon', registry=registry)
nodes_online = Gauge('total_online', 'total online nodes', namespace='gluon', registry=registry)
clients_total = Gauge('clients_total', 'clients total', namespace='gluon', registry=registry)
traffic_total = Gauge('traffic_total', 'traffic total', ['type'], namespace='gluon', registry=registry)
meshvpn = Gauge('meshvpn_count', 'meshvpn', ['peer', 'group'], namespace='gluon', registry=registry)



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
					meshvpn_contime.labels(**deflbl, group=group, peer=p).set(v['established']) # per node
					meshvpn.labels(group=group, peer=p).inc() # per node


		# cpu stats
		for m,v in d['statistics']['stat']['cpu'].items():
			cpu.labels(**deflbl, mode=m).set(v) # per node

		# memory
		memory_usage.labels(**deflbl).set(1-(d['statistics']['memory']['free']/d['statistics']['memory']['total']))
		memory_total.labels(**deflbl).set(d['statistics']['memory']['total'])

		for what,val in d['statistics']['memory'].items():
			memory.labels(**deflbl, type=what).set(val)


		rootfs.labels(**deflbl).set(d['statistics']['rootfs_usage'])
		time.labels(**deflbl).set(d['statistics']['time'])

		process.labels(**deflbl, type='total').set(d['statistics']['processes']['total'])
		process.labels(**deflbl, type='running').set(d['statistics']['processes']['running'])

		domain_counter.labels(**deflbl, domain=d['nodeinfo']['system']['domain_code']).inc()
		batman_adv.labels(**deflbl, compat=d['nodeinfo']['software']['batman-adv']['compat']).inc()



	except KeyError as e:
		eprint(f"{node['nodeid']} has incomplete response. missing {e}. ignore")
		# eprint(json.dumps(node, indent=4, sort_keys=True))
		# exit(0)


# delete_from_gateway(PUSHGATEWAY, 'knoten')
# push_to_gateway(PUSHGATEWAY, job='knoten', registry=registry)

sys.stdout.buffer.write(prometheus_client.generate_latest(registry=registry))
# print(prometheus_client.generate_latest(registry=registry))

eprint("done")
