# Migrating to the new KDL configuration format

Version 2.0 of `rust-motd` includes support for the KDL configuration format.
Support for the legacy TOML configuration is maintained for backwards compatibility,
but it may be removed during a future major release.

Motivation:
- Every component can now appear multiple times in any order
- Controlling the display order with the order of the components in the configuration is now semantically correct and not a trick in how the file is parsed

The following sections show how to migrate each part of the config:

## Global

```toml
[global]
progress_full_character = "="
progress_empty_character = "="
progress_prefix = "["
progress_suffix = "]"
time_format = "%Y-%m-%d %H:%M:%S %Z"
```

```kdl
global {
  version "1.0"
  progress-full-character "="
  progress-empty-character "="
  progress-prefix "["
  progress-suffix "]"
  time-format "%Y-%m-%d %H:%M:%S %Z"
}
```

## Command

```toml
[banner]
color = "red"
command = """
echo '                    __                        __      __  ' &&
echo '   _______  _______/ /_      ____ ___  ____  / /_____/ /  ' &&
echo '  / ___/ / / / ___/ __/_____/ __ `__ \\/ __ \\/ __/ __  / ' &&
echo ' / /  / /_/ (__  ) /_/_____/ / / / / / /_/ / /_/ /_/ /    ' &&
echo '/_/   \\__,_/____/\\__/     /_/ /_/ /_/\\____/\\__/\\__,_/'"""
```

```kdl
command color="red" "
echo '                    __                        __      __  ' &&
echo '   _______  _______/ /_      ____ ___  ____  / /_____/ /  ' &&
echo '  / ___/ / / / ___/ __/_____/ __ `__ \\/ __ \\/ __/ __  / ' &&
echo ' / /  / /_/ (__  ) /_/_____/ / / / / / /_/ / /_/ /_/ /    ' &&
echo '/_/   \\__,_/____/\\__/     /_/ /_/ /_/\\____/\\__/\\__,_/'
"
```

## Weather

```toml
[weather]
loc = "Toronto,Canada"
```

```kdl
weather loc="Toronto,Canada" style="oneline" timeout=10
```

## Service status

```toml
[service_status]
Accounts = "accounts-daemon"
Cron = "cronie"
```

```kdl
service-status {
  service display-name="Accounts" unit="accounts-daemon"
  service display-name="Cron" unit="cronie"
}
```

## Uptime

```toml
[uptime]
prefix = "Up"
```

```kdl
uptime prefix="Uptime"
```

## SSL Certificates

```toml
[ssl_certificates]
sort_method = "alphabetical"

    [ssl_certificates.certs]
    "example.com" = "./cert.pem"
```

```kdl
ssl-certs sort-method="alphabetical" {
  cert name="example.com"  path="./cert.pem"
}
```

## Filesystems

```toml
[filesystems]
root = "/"
```

```kdl
filesystems {
  filesystem name="root" mount-point="/"
  filesystem name="home" mount-point="/home"
}
```

## Memory

```toml
[memory]
swap_pos = "beside"
```

```kdl
memory swap-pos="beside"
```

## Fail2Ban

```toml
[fail_2_ban]
jails = ["sshd"]
```

```kdl
fail2ban {
  jail "sshd"
}
```

## Last Login

```toml
[last_login]
marcel = 2
```

```kdl
last-login {
  user username="marcel" num-logins=2
}
```

## Load Average

```toml
[load_avg]
format = "Load (1, 5, 15 min.): {one:.02}, {five:.02}, {fifteen:.02}"
```

```kdl
load-avg format="Load (1, 5, 15 min.): {one:.02}, {five:.02}, {fifteen:.02}"
```

## Last run

```toml
[last_run]
```

```kdl
last-run
```

## CgStats

```toml
[cg_stats]
state_file = "cg_stats.toml"
threshold = 0.01
```

```kdl
cg-stats state-file="cg_stats.toml" threshold=0.01
```

## Docker

```toml
[docker]
"/nginx-nginx-1" = "Nginx"
```

```kdl
docker {
  container display-name="Nginx" docker-name="/nginx-nginx-1"
}
```
