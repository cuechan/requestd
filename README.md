![pipeline status](https://gitlab.com/cuechan/requestd/badges/master/pipeline.svg)

[Docs](https://cuechan.gitlab.io/requestd/requestd/)

[latest .deb](https://cuechan.gitlab.io/requestd/requestd.deb)

Building
========

You need rust nightly: (assuming you have rust installed via [rustup](https://rustup.rs/))
```
rustup default nightly
```


Help!
=====

"is there a route configured"
-----------------------------

Sometimes it is necessary to explicitely set a route for the multicast
address.

1. get the source ip address for the interface you want to use (`ip a`)
2. add a route for the multicast address:
  `sudo ip route add ff05::2:1001/128 dev <interface> src <source address> table local`
