# `rust-motd`

> Beautiful, useful MOTD generation with zero runtime dependencies

<p align="center">
	<img src="./docs/example_output.svg" />
</p>

I got stuck in dependency hell one too many times
trying to update interpreted alternatives
and decided to write my own MOTD generator in Rust.
The goal of this project is to provide beautiful yet useful status screens
which can quickly give an overview of your server or personal computer.

## Installation

### Building from source

- Install [rustup](https://rustup.rs/) and [cargo](https://github.com/rust-lang/cargo/)
- Install and configure the default toolchain with `rustup install stable` and `rustup default stable`
- Clone this repository and enter it
- Run `cargo build` or `cargo run`

### Arch Linux

`rust-motd` is in the AUR under [`rust-motd-bin`](https://aur.archlinux.org/packages/rust-motd-bin/) thanks to [`cargo-aur`](https://github.com/fosskers/cargo-aur).

## Configuration

`rust-motd` uses a `TOML` configuration file to determine which components to run, and any parameters for those components. Components can be enabled or disabled by including or removing/commenting out the relevant section of configuration. An example configuration file is included in [default_config.toml](default_config.toml).

A configuration file can either be specified as the first argument to `rust-motd` via the comnmand line or placed in one of two default locations. If a config file is not specified as an argument, `rust-motd` will check `$XDG_CONFIG_HOME/rust-motd/config.toml` and `$HOME/.config/rust-motd/config.toml` in that order.

The options for each component are listed below:
### Banner

- `color`: The color of the banner text. Options are black, red, green, yellow, blue, magenta, cyan, white, and light variants of each.
- `command`: A command executed via `sh` which generates the banner. For example, you could pipe the output of `hostname` to `figlet` to generate a block letter banner.

### Weather

The weather component allows you to either specify a [wttr.in](https://wttr.in) url, or a location and display style which will be used to build the url.

Either:

- `url`: a [wttr.in](https://wttr.in) query url for the relevant location. E.g. "wttr.in" or "wttr.in/New+York,New+York?0". For more detail about the options available via the request url, see the [wttr.in documentation](https://github.com/chubin/wttr.in). The response of an http request to the specified url is output directly to the console, so in theory you could use a service other than wttr.in.

or:

- `loc`: The location to retrieve the weather for, e.g. "New York,New York".
- `style`: One of either "oneline", "day", or "full".

In the case both are specified, the `url` parameter is given priority. You can also change the command used to make the http request:

- `command`: Optional, defaults to `curl`. The `url` option is passed as a parameter, so technically you could configure any command you like and use `url` to specify parameters, if you don't want to do a curl request to wttr.in.

### Service Status

 - List of `systemd` services to display the status of. Keys are used as the service display name, while the value is the name of the service itself.

### Uptime

- `prefix`: Text to print before the formatted uptime.

### SSL Certificates

- `sort_method`: The order to sort the displayed ssl certificates. Options are "alphabetical", "expiration", or none, in which case the certs will be displayed in the same order that they appear in the config file.
- `[ssl_certificates.certs]`: A subsection which is a list pairs of of certificate display names (keys) and certificate paths (values).

### Filesystems

 - List of filesystems to print the information of, in the form of pairs of names (used for display) and mount points.

### Fail2Ban

- `jails`: A list of Fail2Ban jails to print the ban amounts of.

### Last Login

- List of users (keys) and number n (values) of that user's n most recent logins to display.

### Last Run

- If present, prints the time that the `rust-motd` was run (useful if updating the motd only periodically e.g. via Cron).

## Setup

### Displaying MOTD on login (server setup)

The canonical MOTD is a message printed on login.
To achieve this, the file `/etc/motd` must be kept up to date with the output of `rust-motd`.
One of the simplest ways to do this is with a cron job.
The line below will update `/etc/motd` every 5 minutes.
This must be run as root (`sudo crontab -e`)
in order to write to the protected file `/etc/motd`.

```cron
*/5 * * * * rust-motd > /etc/motd
```

### Displaying MOTD on every new terminal (personal computer setup)

It can also be nice to show the MOTD locally every time you launch a new terminal emulator
(or on every new pane if you use `tmux`).
Indeed, some components make more sense on a server (ssl, fail2ban, last login)
whereas others make more sense on a local machine (weather, user services).

The setup for this is slightly different.
First of all, you will probably want to run `rust-motd` as your normal user,
not as root.
This is especially true if you are using the user services component.
This also means that you won't have permission to write to `/etc/motd`.
I chose `~/.local/etc/motd`.
Finally, I had to set the environment variable `DBUS_SESSION_BUS_ADDRESS`
in my `crontab` in order to see the status of my user systemd services.
Without it, the underlying call to `systemctl` would return nothing
and nothing would be shown in `rust-motd`.

```cron
*/5 * * * *  export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus; rust-motd > ~/.local/etc/motd
```

Finally, with `~/.local/etc/motd` populated,
the last step is to print the contents of this file every time a new terminal emulator is launched.
Open your shell's configuration file (`.bashrc`, `.zshrc`, etc.)
and add the following line at the very bottom
(if you aliased `cat` to `bat` as I did replace `cat` below with `command cat`):

```
cat $HOME/.local/etc/motd
```


## Alternatives

`rust-motd` took a lot of inspiration from `panda-motd`.

- [panda-motd](https://github.com/taylorthurlow/panda-motd): "a utility for generating a more useful MOTD", written in Ruby

## Acknowledgements

`rust-motd` is made possible by the following packages:

- [wttr.in](https://github.com/chubin/wttr.in) ":partly_sunny: The right way to check the weather"
- [systemstat](https://github.com/unrelentingtech/systemstat): "Rust library for getting system information", used for filesystem usage
- [termion](https://docs.rs/termion/1.5.6/termion/): Rust library used to print fancy colours in the terminal
- [termtosvg](https://github.com/nbedos/termtosvg): "Record terminal sessions as SVG animations", used to generate the preview in the README
- [bytesize](https://docs.rs/bytesize/1.0.1/bytesize/): Rust library used for binary size representations
- [humantime](https://docs.rs/humantime/2.0.1/humantime/): "Human-friendly time parser and formatter", used for uptime component
