# Bag &emsp; [![Build Status]][travis] ![License]

[Build Status]: https://img.shields.io/travis/flight-rs/bag/master.svg?style=flat-square
[travis]: https://travis-ci.org/flight-rs/bag
[License]: https://img.shields.io/github/license/flight-rs/bag.svg?style=flat-square

**Bag is badass declarative asset loader and bundler for Rust.**
---

```rust
bag!{static LOGO: TryBag<RgbaImage> = "image/logo.png"};
bag!{static LICENCE: TryBag<str> = "LICENSE"};
```

```toml
[alias]
"image/**" = "public/img/**"

[runtime.alias]
"image/**" = "assets/**"

[target."cfg(wasm)"]
static = true

[asset."LICENSE"]
static = true

[format."image::RgbaImage"]
loader = "bag_image::LoadImageBuffer"
```

None of the above actually works yet, but I can dream.