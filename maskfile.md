# Marmite Development Tasks

This file contains development tasks for the Marmite static site generator.

## build

> Build the release binary

~~~bash
cargo build --release
~~~

## serve

> Build and serve the example site while watching for changes on the example dir

~~~bash
rm -rf ./example/site && cargo run --quiet -- example --serve --watch --force -vvvv
~~~

## serve_site

> Build and serve the marmite.blog site locally including the CI customizations

~~~bash
.github/prepare_site.sh
python .github/contributors.py marmitesite/content/contributors.md
cargo run --quiet -- marmitesite --serve --watch --force -vvvv
~~~

## serve_theme

> Build and serve with theme_template

~~~bash
rm -rf marmitesite
cp -R example marmitesite
rm -rf marmitesite/content/_hero.md
cargo run --quiet -- marmitesite --serve --watch --force -vvvv --theme theme_template
~~~

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

> Watch for changes on the whole source code  and rebuild the example site without serving it.

~~~bash
cargo watch -c -q -x "run example --force -vvvv"
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

## retag (tag)

> Amend changes and retag and push tags again

~~~bash
git push origin :refs/tags/$tag
git commit --amend --no-edit --allow-empty
git tag -f $tag
git push --force-with-lease
git push origin $tag
~~~

## coverage

> Calculate code coverage and generate cobertura.xml

~~~bash
cargo +nightly tarpaulin --verbose --all-features --workspace --timeout 120 --out xml
~~~
