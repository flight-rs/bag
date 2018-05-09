# <img src="/media/logo+name.svg" height="64"> &emsp; [![Build Status]][travis] ![License]

[Build Status]: https://img.shields.io/travis/flight-rs/bag/master.svg?style=flat-square
[travis]: https://travis-ci.org/flight-rs/bag
[License]: https://img.shields.io/github/license/flight-rs/bag.svg?style=flat-square

**Grab is a declarative asset loader and bundler for Rust.**
---

```rust
fn main<P: Pack>(pack: &mut P) {
    let banner = grab!(pack, { -"static", compress = "jpg" }, "image/banner.png" => ImageBuffer);
    let license = grab!(pack, "static", "LICENSE" => &str);
    let logo = grab!(pack, "image/logo.png" => ImageBuffer);
}
```

None of the above actually works yet, but I can dream.
