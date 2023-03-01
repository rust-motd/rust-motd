## Creating a release

1. Update the version in [`Cargo.toml`](./Cargo.toml)
1. Create a git tag
1. Run `cargo bundle --release`
1. Run `cargo aur`
1. Add `-archlinux` after the version in the resulting file
1. Move `PKGBUILD` to the AUR repo and create a commit there
1. Create `sha256` checksums of those binary files
1. Create a release in GitHub based on the tag
1. Upload binaries
1. Write a changelog
1. Publish!
