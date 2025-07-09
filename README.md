# `rust-motd`

> Beautiful, useful, configurable MOTD generation with zero[¹](#footnote-1) runtime dependencies

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
- Install the equivalent of the `libssl-dev` package using your package manager
- Clone this repository and enter it
- Run `cargo build` or `cargo run`

<a id="compiling-alpine"></a>
Note: To cross compile, you may need to install additional packages. For example, to cross compile for Alpine, it was necessary to install the `musl-tools` package on Ubuntu (specifically to compile the `ring` crate), after which an executable could be successfully cross-compiled with `cargo build --target x86_64-unknown-linux-musl` (assuming you've already added the `musl` toolchain via `rustup target add x86_64-unknown-linux-musl`).
[See more.](https://www.reddit.com/r/rust/comments/qdm8gf/comment/hhor67v/?utm_source=share&utm_medium=web2x&context=3)

### Arch Linux

`rust-motd` is in the AUR under [`rust-motd-bin`](https://aur.archlinux.org/packages/rust-motd-bin/) thanks to [`cargo-aur`](https://github.com/fosskers/cargo-aur).

### Debian and derivatives

- You can install `rust-motd-deb` with [pacstall](https://github.com/pacstall/pacstall).
- There is a `.deb` file available in the [Releases](https://github.com/rust-motd/rust-motd/releases) tab.

### NixOS

`rust-motd` is available in the [`nix`](https://nixos.org/) package manager under the name [`rust-motd`](https://search.nixos.org/packages?show=rust-motd&from=0&size=50&sort=relevance&type=packages&query=rust-motd). Unlike the other formats, this is not packaged by the authors of `rust-motd`.

## Configuration

`rust-motd` is configured using a [KDL](https://kdl.dev/) file.
Support for the legacy TOML configuration is maintained for backwards compatibility,
but it may be removed during a future major release.
The rest of this section will assume the new KDL format.
See [the migration guide](./docs/migration.md) for details.

The most basic configuration is given below. `global.version` must be specified and the value must be `1.0`. The components (described in the next sections) will be displayed in the order they appear inside of `components {}`. Each component can occur as many times as you'd like, with different parameters each time. An example configuration file [example_config.kdl](example_config.kdl).
```kdl
global {
  version "1.0"
}
components {
}
```

A configuration file can either be specified as the first argument to `rust-motd` via the command line or placed in one of two default locations. If a config file is not specified as an argument, `rust-motd` will check `$XDG_CONFIG_HOME/rust-motd/config.kdl` and `$HOME/.config/rust-motd/config.kdl` in that order.

The options for each component are listed below.
Each section lists children, properties, and attributes.
Components that take children show multiple pieces of information:
```kdl
components {
  component-with-children {
    child
    child
  }
}
```

If a component (or a child of a component) takes properties, put the name of the property, an equals sign (`=`), and the value.
```kdl
components {
  component property-name="value"
}
```

If a component (or a child of a component) takes arguments, do not put the argument name or equals sign, just place the value.
```kdl
components {
  command "echo this is the command to run"
}
```

### Command (formerly Banner)

Display the output of a command (executed via `sh`).

Example:
```kdl
command color="red" "hostname | figlet -f slant"
```

Arguments:
- Command: The command to run. Essentially the argument to `sh -c`.

Properties:
- `color`: The color of the banner text. Options are black, red, green, yellow, blue, magenta, cyan, white, and light variants of each. The default is white.

### Weather

The weather component allows you to either specify a [wttr.in](https://wttr.in) URL, or a location and display style which will be used to build the URL.

Example:
```kdl
weather loc="Toronto,Canada" style="oneline" timeout=10
```

Properties:
- `url`: a [wttr.in](https://wttr.in) query URL for the relevant location. E.g. `https://wttr.in` or `https://wttr.in/New+York,New+York?0`. For more detail about the options available via the request URL, see the [wttr.in documentation](https://github.com/chubin/wttr.in). The response of an http request to the specified URL is output directly to the console, so in theory you could use a service other than [wttr.in](wttr.in). If unspecified, the URL is automatically built from the following properties.
- `loc`: The location to retrieve the weather for, e.g. "New York,New York". If `url` is specified, this has no effect. If unspecified, [wttr.in](https://wttr.in) will try to determine your location automatically.
- `style`: One of either "oneline", "day", or "full". If `url` is specified, this has no effect. The default is "day".
- `user-agent`: User-Agent to use when connecting. The default is `curl`.
- `proxy`: The http proxy server which used to access internet.
- `timeout`: Timeout in seconds for the network request. The default is `5`.

### Service Status

Displays the status of `systemd` services.

Example:
```kdl
service-status {
  service display-name="Accounts" unit="accounts-daemon"
  service display-name="Cron" unit="cronie"
}
```

Children:
- `service`: Specify once for each service

Properties of `service`:
- `unit`: The name of the unit. Basically corresponds to the argument to `systemctl` commands.
- `display-name`: The display name. Can be anything. For example, unit is called `accounts-daemon`, you may want to display `Accounts`.

### User Service Status

Displays the status of `systemd` services.

Example:
```kdl
user-service-status {
  service display-name="gpg-agent" unit="gpg-agent"
}
```

### Docker Status

Displays the status of docker containers.

Example:
```kdl
docker {
  container display-name="Nginx" docker-name="/nginx-nginx-1"
  container display-name="MariaDB" docker-name="/mariadb-mariadb-1"
}
```

Properties:
- `socket`: The socket to use. Allows connecting to the Podman daemon instead of Docker, for example, by setting `socket="unix:///run/user/1000/podman/podman.sock"`. The default is the docker socket.
- `title`: Allows changing the title of the component. For example, you may want to change it to `Podman` if you use the Podman socket. The default is `Docker`.

Children:
- `container`: Specify once for each service.

Properties of `container`:
- `display-name`: The pretty name of the container.
- `docker-name`: The internal docker name (`NAMES` column of `docker ps`). Containers can have multiple names, and the container is selected if any of the names match.
The key **must** start with a `/` for internal containers (please see [here](https://github.com/moby/moby/issues/6705)).

### Docker Compose Status

Displays the status of all docker containers in a compose stack.

Example:
```kdl
docker-compose {
  stack display-name="Nginx" path="~/docker/nginx"
}
```

Properties:
- `socket`: The socket to use. Allows connecting to the Podman daemon instead of Docker, for example, by setting `socket="unix:///run/user/1000/podman/podman.sock"`. The default is the docker socket.
- `title`: Allows changing the title of the component. For example, you may want to change it to `Podman` if you use the Podman socket. The default is `Docker`.

Children:
- `stack`: Specify once for each stack.

Properties of `stack`:
- `display-name`: The pretty name of the stack.
- `path`: The path to the parent directory of the `docker-compose.yml` file. Supports `~` expansion.

### Uptime

Displays the uptime.

Example:
```kdl
uptime prefix="Uptime"
```

- `prefix`: Text to print before the formatted uptime. The default is `Up`.

### SSL Certificates

Shows the expiration of SSL certificates.

Example:
```kdl
ssl-certs sort-method="alphabetical" {
  cert name="example.com"  path="./cert.pem"
}
```

Properties:
- `sort-method`: The order to sort the displayed SSL certificates. Options are "alphabetical", "expiration", or "manual", in which case the certs will be displayed in the same order that they appear in the config file.

Children:
- `cert`: Specify once for each certificate.

Properties of `cert`:
- `name`: The pretty name to display.
- `path`: Path to the certificate file. If using LetsEncrypt, this should be `cert.pem`, not `privkey.pem`.

### Filesystems

Displays information about filesystems and a bar showing the used space.

Example:
```kdl
filesystems {
  filesystem filesystem-name="root" mount-point="/"
  filesystem filesystem-name="home" mount-point="/home"
}
```

Children:
- `filesystem`: Specify once for each filesystem.

Properties of `filesystem`:
- `name`: Display name for the filesystem.
- `mount-point`: The directory where the filesystem is mounted, used to identify it.

### Memory

Displays information about used memory.

Example:
```kdl
memory swap-pos="beside"
```

Properties:
 - `swap-pos`: Either `beside`, `below` or `none` to indicate the location to display the swap memory usage, if any.

### Fail2Ban

Shows information about Fail2Ban jails.

Example:
```kdl
fail2ban {
  jail "sshd"
}
```

Children:
- `jail`: Specify once for each jail.

Arguments of `jail`:
- `name`: The name of the jail.

### Last Login

Displays the last logins to the machine.

Example:
```kdl
last-login {
  user username="marcel" num-logins=2
}
```

Children:
- `user`: Specify once for each user.

Properties of `user`:
- `username`: Username of the user.
- `num-logins`: The number of logins to display.

### Last Run

Displays the timestamp of the last time `rust-motd` was run. This is useful if updating the motd only periodically e.g. via Cron.

Example:
```kdl
last-run
```

### Load Average

Displays information about the current system load.

Example:
```kdl
load-avg format="Load (1, 5, 15 min.): {one:.02}, {five:.02}, {fifteen:.02}"
```

Properties:
- `format`: Format of the printed message. Can contain specifiers for
  parameters `one`, `five` and `fifteen` representing different load
  average values.
- `warn-threshold`: Optional threshold for printing load values in
  yellow. Defaults to the number of CPUs in the system.
- `bad-threshold`: Optional threshold for for printing load values in
  red. Defaults to four times the number of CPUs in the system.

### Cgroup Statistics

Prints CPU usage by users and services since the last invocation. The
numbers are based on statistics from systemd-managed cgroups.

Example:
```kdl
cg-stats state-file="cg_stats.toml" threshold=0.01
```

Properties:
- `state-file`: File name where to store cgroup statistics for the next invocation.
- `threshold`: Number in range [0.0, 1.0]. Output lines are generated
  only for cgroups with CPU usage higher than this value.

### Global Config
The global configuration is used for settings that may span multiple components, e.g. the time format string, and progress bar style.

- `version`: Must be specified and must be `1.0`.
- `progress-full-character` (Default `"="`): The character to use for the line segment of the progress bar indicating the "active" portion of the quantity represented
- `progress-empty-character` (Default `"="`): The character to use for the line segment of the progress bar indicating the "inactive" portion of the quantity represented
- `progress-prefix` (Default `"["`): The character to used to cap the left side of the progress bar
- `progress-suffix` (Default `"]"`): The character to used to cap the right side of the progress bar
- `progress-width` (Default `80`): The default width of the progress bar, used only if no other "size hint" is available. More specifically, the `filesystem` component will automatically determine its width. If the `filesystem` component is present, then the `memory` component will use the width of the filesystem as its size hint. Otherwise it will use the configured value.
- `time-format` (Default `"%Y-%m-%d %H:%M:%S %Z"`): time format string

## Setup

### Updating periodically with chron

This is the recommended setup for the fastest performance opening a new shell.

#### Displaying MOTD on login (server setup)

The canonical MOTD is a message printed on login.
To achieve this, the file `/etc/motd` must be kept up to date with the output of `rust-motd`.
One of the simplest ways to do this is with a cron job.
The line below will update `/etc/motd` every 5 minutes.
This must be run as root (`sudo crontab -e`)
in order to write to the protected file `/etc/motd`.

```cron
*/5 * * * * rust-motd > /etc/motd
```

#### Displaying MOTD on every new terminal (personal computer setup)

It can also be nice to show the MOTD locally every time you launch a new terminal emulator
(or on every new pane if you use `tmux`).
Indeed, some components make more sense on a server (`ssl`, `fail2ban`, `last_login`)
whereas others make more sense on a local machine (weather, user services).

The setup for this is slightly different.
First of all, you will probably want to run `rust-motd` as your normal user,
not as root.
This is especially true if you are using the user services component.
This also means that you won't have permission to write to `/etc/motd`.
I chose `~/.local/etc/motd`.
Finally, I had to set the environment variable `DBUS_SESSION_BUS_ADDRESS`
in my `crontab` in order to see the status of my user `systemd` services.
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

### Updating on every new shell

This setup prioritizes up-to-date data above shell performance.
It should only be used if `rust-motd` is very fast with your configuration.
Some components can take considerable time (weather has to hit the internet).
You will experience this delay on every new shell.

#### With shellrc

The simplest way to show `rust-motd` with fresh data on every new shell
is simply to add the command `rust-motd` to your shell configuration file (`~/.bashrc`, `~/.zshrc`, ...).

#### With update-motd (Ubuntu only)

`update-motd` is Ubuntu's standard method for dynamically updating a MOTD (see [update-motd(5)](https://manpages.ubuntu.com/manpages/jammy/man5/update-motd.5.html)).

You can create a script like this (`/etc/update-motd.d/99-rust-motd`) :

```bash
#!/usr/bin/env bash
rust-motd /etc/rust-motd.toml
```

Don't forget to make the script executable.

> [!IMPORTANT]
> `$HOME` is not defined when this script is executed, so you need to pass the config as a parameter to avoid the error: `Config Error: environment variable not found.`

#### With PAM

This is the most "Linux standard" solution.

Place the execution of an optional script (not affecting authentication in case of failure) before the execution of the pam module `pam_motd.so`.
```pam
session optional pam_exec.so /usr/local/bin/update-motd
```
And in the script `/usr/local/bin/update-motd`, put:
```bash
#!/usr/bin/env bash
rust-motd /etc/rust-motd.toml > /etc/motd
```

> [!IMPORTANT]
> `$HOME` is not defined when this script is executed, so you need to pass the config as a parameter to avoid the error: `Config Error: environment variable not found.`

## Alternatives

`rust-motd` took a lot of inspiration from `panda-motd`.

- [panda-motd](https://github.com/taylorthurlow/panda-motd): "a utility for generating a more useful MOTD", Ruby
- [motd-on-acid](https://github.com/x70b1/motd-on-acid):  "This MOTD has so many colors!", Shell
- [fancy-motd](https://github.com/bcyran/fancy-motd): "Fancy, colorful MOTD written in bash. Server status at a glance.", Shell
- [HermannBjorgvin/MOTD](https://github.com/HermannBjorgvin/motd): "Mini MOTD, a customizable, configurable, standardized MOTD for your homelab server or laptop", Shell

Search "MOTD" on [r/unixporn](https://reddit.com/r/unixporn) for more!

## Acknowledgements

A huge thank you to the kind folks at Jupiter Broadcasting
for featuring `rust-motd` on [Linux Unplugged 428](https://linuxunplugged.com/428)!

`rust-motd` is made possible by the following packages:

- [wttr.in](https://github.com/chubin/wttr.in) ":partly_sunny: The right way to check the weather"
- [systemstat](https://github.com/unrelentingtech/systemstat): "Rust library for getting system information", used for filesystem usage
- [termion](https://docs.rs/termion/1.5.6/termion/): Rust library used to print fancy colours in the terminal
- [termtosvg](https://github.com/nbedos/termtosvg): "Record terminal sessions as SVG animations", used to generate the preview in the README
- [bytesize](https://docs.rs/bytesize/1.0.1/bytesize/): Rust library used for binary size representations
- [humantime](https://docs.rs/humantime/2.0.1/humantime/): "Human-friendly time parser and formatter", used for uptime component

## Footnotes
<a id="footnote-1"></a>
¹: Certain components do have dependencies: `fail2ban` (`fail2ban`), `service_status` (`systemd`),
`last_login` (`last`).
However, it would not make sense to request the status of a package that is not installed.
[Furthermore, there are some caveats when compiling for minimal distributions like Alpine Linux.](#compiling-alpine)
