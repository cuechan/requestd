# requestd

[![pipeline status](https://gitlab.com/cuechan/requestd/badges/master/pipeline.svg)](https://gitlab.com/cuechan/requestd/-/commits/master)
[![docs](https://img.shields.io/badge/Docs-here-blue)](https://cuechan.gitlab.io/requestd/requestd/)
[![latest .deb](https://img.shields.io/badge/Debian-requestd.deb-%23CE0056)](https://cuechan.gitlab.io/requestd/requestd.deb)


Building
========

`sudo apt install libzmq3-dev libsqlite3-dev git automake pkg-config libssl-dev libjq-dev libonig-dev libtool cmake`

You need rust nightly: (assuming you have rust installed via [rustup](https://rustup.rs/))
```
rustup default nightly
```

Run
===

`requestd --config assets/config.yaml`


Endpoints
=========

You can configure different endpoints which distribute the respondd responses via different protocols. Currently there are 3 implemented protocols:

- http
- mqtt
- [zmq pub/sub](https://zeromq.org/socket-api/#publish-subscribe-pattern)


http
----
To enable the **http** endpoint add the following to your `requestd.yml`:

```yaml
web:
  listen: "[::]:21001"
```

All node responses will be available at `http://localhost:21001/responses`



mqtt
----
To enable the **mqtt** endpoint add the following to your `requestd.yml`:

```yaml
mqtt:
  broker: localhost:1883
  topic: ffhl
```

Mqtt authentication in currently not supported.


zmq
---
To enable the **zmq** endpoint add the following to your `requestd.yml`:


```yaml
zmq:
  bind_to: "tcp://*:21002"
```

You can now `SUB`scribe to this endpoint wiht another aplication. Remember that zmq pub/sub also uses topics. The topic used by requestd is `requestd`. For each message you need to call `zmq_recv()` twice. The first call will receive the topic, the second will receive the actual message.


Help!
=====

## "is there a route configured?"

Sometimes it is necessary to explicitely set a route for the multicast
address. There a two ways to set up a route:

### 1. use `ip r` for temprary setups
1. get the source ip address for the interface you want to use (`ip a`)
2. add a route for the multicast address:
  `sudo ip route add ff05::2:1001/128 dev <interface> src <source address> table local`

### 2. use systemd-networkd
If your Host use systemd-network for the network configuration you can simply
add this to the `.network` file for the interface connected to the freifunk network:

```
[Match]
# use your iface name here
Name=ffhl

[Network]
# some network config here
# https://www.freedesktop.org/software/systemd/man/systemd.network.html#%5BNetwork%5D%20Section%20Options

# finally an explicit route entry:
[Route]
Destination=ff05::2:1001/128
Type=multicast
Table=local
```


"I am behind a freifunk node that has ebtable filters"
------------------------------------------------------

https://wiki.luebeck.freifunk.net/docs/infrastruktur/gallifrey-gluon01/

```
ssh root@node.ffhl
echo "rule 'MULTICAST_OUT -p IPv6 --ip6-protocol udp --ip6-destination-port 1001 --ip6-dst ff02::2:1001 -j RETURN'" > /lib/gluon/ebtables/110-mcast-allow-respondd
echo "rule 'MULTICAST_OUT -p IPv6 --ip6-protocol udp --ip6-destination-port 1001 --ip6-dst ff05::2:1001 -j RETURN'" > /lib/gluon/ebtables/110-mcast-allow-respondd
/etc/init.d/gluon-ebtables restart
```
