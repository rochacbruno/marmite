check:
    cargo fmt -- --check
    cargo clippy

pedantic:
    cargo fmt -- --check
    cargo clippy -- -W clippy::pedantic

fmt:
    cargo fmt

fix:
    cargo clippy --fix

pedantic_fix:
    cargo clippy --fix -- -W clippy::pedantic

[doc('Bump version in Cargo.toml')]
[group('maintainer')]
bumpversion VERSION:
    #!/usr/bin/env bash
    VERSION="{{ trim_start_match(VERSION, "v") }}"
    shopt -s expand_aliases
    alias set-version='cargo set-version --package marmite --locked'
    set-version "$VERSION" || cargo install -y cargo-edit@0.13.0 && set-version "$VERSION"
    cargo generate-lockfile
    just fmt
    git add ./Cargo.toml ./Cargo.lock
    git commit -m "chore: bump version to $VERSION"

[doc('Push a new tag to the repository')]
[group('maintainer')]
pushtag TAG:
    #!/usr/bin/env bash
    VERSION="{{ trim_start_match(TAG, "v") }}"
    git push origin :refs/tags/$VERSION # delete the origin tag
    git tag -d $VERSION # delete the local tag
    git tag -a "$VERSION" -m "chore: push $VERSION"
    git push origin $VERSION

[doc('Publish a new version (bumpversion + pushtag)')]
[group('maintainer')]
publish TAG:
    just bumpversion {{ TAG }}
    just pushtag {{ TAG }}
