# Changelog

## [2.0.0] 2025-07-09

- Add support for KDL configuration (legacy TOML support is maintained) (#37)
- Rename "banner" component to "command"
- Add support for docker compose component (#30)
- Add support for using a different Docker API socket (#34)

## [1.1.1] 2025-06-18

- Update dependencies

## [1.1.0] 2025-03-12

- Add timeout for weather
- Order filesystems, containers, logins, certs, and services the same as the config (#31)
- Add load average component (#32)
- Add cg_stats component (#33)

## [1.0.1] 2023-03-01

- Support configuring the user agent when making requests to wttr.in
- Start packaging the program as a `.deb`

## [1.0.0] 2022-10-06

The major version bump is because the change to the configuration file can be
considered a breaking change. If you do not order your configuration file the
same as the previous hard-coded order they were printed, the order will now
change, which can be considered a breaking change.

- Allow reordering components (6daa15d869127e63cbd1c61ff9bc538e47208d3d)
  You can now change the order in which the components are printed by
  changing the order in which they are defined in the configuration file
- Fix bug with the layout of memory component (4d3be74ca226b089a03f6de65bae20c48f01d277)
  There were some math mistakes making the side-by-side progress bars
  not always line up with the full width bar (used by filesystems, for
  example). Also, code cleanup
- Code cleanup (983cb0e9ec763dc13d1acc766a48d59df5dd03e0)
- `last-rs` dependency updated and now in crates.io (4dd13ef8d593125342cc5935d66fc8235f9ede87)
- Update rust version (310bf1bc7cc1af884c9e435f2cce648bac20f526) and
  dependencies (591a84e6acbf2e956e026ebb23f1cbb54def4dd5)

## [0.2.1] 2022-04-04

- Fix build on Darwin (issue #20)

## [0.2.0] 2022-01-04

- Add docker component (d9fdf32)
- Revamp last_login component (parse utmp directly) (05f91af)
- Add memory component for RAM and swap (8adbf1a)
- Support proxy for weather component's network request (d5bbe36)
- More small improvements

## [0.1.1] 2021-10-16

- Add configuration section of README (78ebd6c)
- Add different sorting methods to SSL certs (efd52e3 and #4)
- Remove runtime dependencies curl (c50b9dd) and openssl (127e3e8)
- Other clean ups and improvements

## [0.1.0] 2021-05-30

- Initial release
