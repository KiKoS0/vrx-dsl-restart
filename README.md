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

### Install as a procd service

```bash
> cp procd /etc/init.d/dsl_restart
> /etc/init.d/dsl_restart enable

> # Should return the service symlink
> ls -la /etc/rc.d/S* | grep dsl_restart
21 Jun 13 18:44 /etc/rc.d/S95dsl_restart -> ../init.d/dsl_restart

> /etc/init.d/dsl_restart start
> /etc/init.d/dsl_restart status
running
```

#### Example procd init script: https://openwrt.org/docs/guide-developer/procd-init-script-example

### Force a DSL restart
```bash
> # Force trigger a reboot 
> echo "vrx518_tc:ptm_showtime_exit" | tee /dev/kmsg

```