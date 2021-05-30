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

## Configuration

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
