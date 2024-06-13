Automates unloading and reloading of the DSL VRX518 kernel module on Fritzbox 7530 to fix connectivity issues caused by DSL line microcuts.

```bash
> root@OpenWrt:~ uname -m
armv7l
```

### Cross compiling rust to `armv7l`

```bash
> sudo apt install gcc-arm-linux-gnueabi

> cargo install cross

> cross build --target armv7-unknown-linux-gnueabi --release
```