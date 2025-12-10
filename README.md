# Get Proxy by Network

Based off of the example code at [Alfablos/rust-dbus-wifi](https://github.com/Alfablos/rust-dbus-wifi), though almost completely rewritten.

## Structure

* `network_manager`: a proxy implementation for `org.freedesktop.NetworkManager`
* `activeconnections`: a proxy implementation for `org.freedesktop.NetworkManager.Connection.Active`

Both were generated via `zbus-xmlgen`. To regenerate:

```shell
cargo install zbus-xmlgen
zbus-xmlgen system org.freedesktop.NetworkManager /org/freedesktop/NetworkManager
zbus-xmlgen system org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/ActiveConnection/10
```

## Build
```shell
cargo build --release
```

## Run
Needs a config file that looks like this:

```toml
["Network Name"]
http = "http://10.0.0.1:3142/"
```

It will first check `~/.config/net.cdslash.proxy-by-net/config.toml`; if that file doesn't exist, it will check `/etc/proxy-by-network.toml`.

Remember to put quotes around your network name if it has spaces in it, or maybe always.

When run, will:
1. Get the active default connection from NetworkManager
2. Check for the name of the connection
3. Look for that configuration in the config file
4. Print the `http` key to stdout (in future: detect if the user wants http or https and print accordingly)

The program will log to stderr if it's a terminal or journald if not.

## Apt Config

If you want to use this to automatically provide apt with an apt proxy configuration (which is what I wrote this for), add a file in `/etc/apt/apt.conf.d/` with similar contents:

```
Acquire::http::Proxy-Auto-Detect "/some/path/to/proxy-by-network";
```

Apt will pass the proxy URL to the program on the command-line, but currently we don't do anything with it, e.g. filtering based on the URL. We just return the same proxy for everything.

## Reference

* [Alfablos' rust-dbus-wifi repo](https://github.com/Alfablos/rust-dbus-wifi)
* [zbus](https://docs.rs/zbus/latest/zbus/)
* [zbus-xmlgen](https://docs.rs/crate/zbus_xmlgen/)
