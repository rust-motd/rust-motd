## Creating a release

1. Update the version in [`Cargo.toml`](./Cargo.toml)
1. Create a git tag
1. Run `cargo bundle --release`
1. Run `cargo aur` (TODO: I think this is run from the other repository (the one pushed to the AUR with PKGBUILD and such))
1. Create `sha256` checksums of those binary files
1. Create a release in GitHub based on the tag
1. Upload binaries
1. Write a changelog
1. Publish!
