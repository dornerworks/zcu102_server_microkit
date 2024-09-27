# ZCU102 - seL4 Microkit IP Server

This repository demonstrates the use of the [seL4 crates](https://github.com/seL4/rust-sel4) with
the [seL4 Microkit](https://github.com/seL4/microkit).

This example also utilizes the [HAL rust drivers](https://github.com/dornerworks/zynqmp_hal) for the Zynq UltraScale+ MPSoC (ZUS+). 

The server system consists of two components:

- `eth-driver` (untrusted): Ethernet driver that takes advantage of the ZUS+ rust HAL and `smoltcp` traits to standardize interaction.
- `ping` (untrusted): Sets up the `smoltcp` network stack and responds to ARP and ping requests.

### Rustdoc for the `sel4-microkit` crate

https://sel4.github.io/rust-sel4/views/aarch64-microkit/aarch64-sel4-microkit/doc/sel4_microkit/index.html

### Quick start

The only requirements for getting started are Git, Make, and Docker.

First, clone this repository. Then build, run, and enter a Docker container for development:

```
make -C docker/ run && make -C docker/ exec
```

Inside the container, build and emulate the demo:

```
make
```

This creates a file `build/loader.img` which can be run on a ZCU102 development board.

Assuming using U-Boot with TFTP, the following command can be run:

```
dhcp; tftpboot 0x40000000 loader.img; go 0x40000000
```
