{# Reusable installation instructions for embedding in content pages #}
The quickest way to install Marmite is using the install script:

```bash
curl -sS https://marmite.blog/install.sh | sh
```

Alternatively, if you have Rust installed, you can use cargo:

```bash
cargo binstall marmite
```
or
```bash
cargo install marmite
```

Or install with pip/uvx:

```bash
pip install marmite
# OR
uvx marmite
```

Or download the pre-built **binary** from the [releases](https://github.com/rochacbruno/marmite/releases)

<details>
<summary>Or install from a package manager</summary>

**Homebrew (macOS/Linux)**
```console
brew install marmite
```
[View formula](https://formulae.brew.sh/formula/marmite)

**Arch Linux**
```console
pacman -S marmite
```
[View package](https://archlinux.org/packages/extra/x86_64/marmite/)

**FreeBSD**
```console
pkg install marmite
```
[View port](https://www.freshports.org/www/marmite/)

</details>

<details>
<summary>Or use Docker</summary>

```console
docker run -v $PWD:/input ghcr.io/rochacbruno/marmite
```

To serve locally:
```console
docker run -p 8000:8000 -v $PWD:/input ghcr.io/rochacbruno/marmite --serve
```

</details>

For more details see the [[installation]] guide.
