![pipeline status](https://gitlab.com/cuechan/requestd/badges/master/pipeline.svg)

[Docs](https://cuechan.gitlab.io/requestd/requestd/)

[latest .deb](https://cuechan.gitlab.io/requestd/requestd.deb)

Building
========

`sudo apt install libzmq3-dev libsqlite3-dev`

You need rust nightly: (assuming you have rust installed via [rustup](https://rustup.rs/))
```
rustup default nightly
```

Run
===

`requestd --config assets/config.yaml`

Help!
=====

"is there a route configured"
-----------------------------

Sometimes it is necessary to explicitely set a route for the multicast
address.

1. get the source ip address for the interface you want to use (`ip a`)
2. add a route for the multicast address:
  `sudo ip route add ff05::2:1001/128 dev <interface> src <source address> table local`

"I am behind a freifunk node that has ebtable filters"
------------------------------------------------------

https://wiki.luebeck.freifunk.net/docs/infrastruktur/gallifrey-gluon01/

```
ssh root@node.ffhl
root@node:/# echo "rule 'MULTICAST_OUT -p IPv6 --ip6-protocol udp --ip6-destination-port 1001 --ip6-dst ff02::2:1001 -j RETURN'
> rule 'MULTICAST_OUT -p IPv6 --ip6-protocol udp --ip6-destination-port 1001 --ip6-dst ff05::2:1001 -j RETURN'" > /lib/gluon/ebtables/110-mcast-allow-respondd
root@node:/# /etc/init.d/gluon-ebtables restart
```
