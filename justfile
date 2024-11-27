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

watch:
    cargo watch -c -q -x "run example ../site_example -vvvv"

[doc('Bump version in Cargo.toml')]
[group('maintainer')]
bumpversion TAG:
    #!/usr/bin/env bash
    cargo set-version --version || cargo install -y cargo-edit@0.13.0
    cargo set-version --package marmite --locked "{{ TAG }}"
    cargo generate-lockfile
    just fmt
    git add ./Cargo.toml ./Cargo.lock
    git commit -m "chore: bump version to {{ TAG }}"

[doc('Push a new tag to the repository')]
[group('maintainer')]
pushtag TAG:
    #!/usr/bin/env bash
    git tag -a "{{ TAG }}" -m "chore: push {{ TAG }}"
    git push --tags

[doc('Publish a new version (bumpversion + pushtag)')]
[group('maintainer')]
publish TAG:
    just bumpversion {{ TAG }}
    just pushtag {{ TAG }}
