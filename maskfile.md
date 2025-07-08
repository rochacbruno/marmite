# Marmite Development Tasks

This file contains development tasks for the Marmite static site generator.

## check

> Check code formatting and run clippy

~~~bash
cargo fmt -- --check
cargo clippy
~~~

## pedantic

> Check code formatting and run clippy with pedantic warnings

~~~bash
cargo fmt -- --check
cargo clippy -- -W clippy::pedantic
~~~

## fmt

> Format the code

~~~bash
cargo fmt
~~~

## fix

> Fix clippy warnings

~~~bash
cargo clippy --fix
~~~

## pedantic_fix

> Fix clippy warnings with pedantic settings

~~~bash
cargo clippy --fix -- -W clippy::pedantic
~~~

## watch

> Watch for changes and rebuild

~~~bash
cargo watch -c -q -x "run example ../site_example -vvvv"
~~~

## bumpversion (tag)

> Bump version in Cargo.toml

~~~bash
#!/usr/bin/env bash
cargo set-version --version || cargo install -y cargo-edit@0.13.0
cargo set-version --package marmite --locked "$tag"
cargo generate-lockfile
mask fmt
git add ./Cargo.toml ./Cargo.lock
git commit -m "chore: bump version to $tag"
~~~

## pushtag (tag)

> Push a new tag to the repository

~~~bash
#!/usr/bin/env bash
git tag -a "$tag" -m "chore: push $tag"
git push --tags
~~~

## publish (tag)

> Publish a new version (bumpversion + pushtag)

~~~bash
mask bumpversion "$tag"
mask pushtag "$tag"
~~~