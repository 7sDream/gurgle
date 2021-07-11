# Gurgle

[![Badge with github icon][github-badge-img]][github-home] [![Badge with document icon][doc-badge-img]][doc-home]

Gurgle is yet another dice rolling crate using TRPG-like syntax.

## Have a Taste

```rust
let attack = "3d6+2d4+1";

println!("roll your attack({}), result: {}", attack, gurgle::roll(attack).unwrap());

// output: roll your attack(3d6+2d4+1), result: 16
```

```rust
use gurgle::Gurgle;

let attack = "3d6+2d4+1>15";
let dice = Gurgle::compile(attack).unwrap();
let result = dice.roll();

println!("roll your attack({}), result: {}", attack, result);

// output: roll your attack(3d6+2d4+1>15), result: (4+3+1) + (1+3) + 1 = 15, target is >15, failed
```

Notice: `Display` trait for rolling result is implemented only if feature `detail`(which is enabled by default) is enabled.

But you can always use `result.value()` to get rolling result value(i64), and `result.success()` to get if it's a success.

See [docs][doc-home] for full syntax and example.

## License

BSD 3-Clause Clear License, See LICENSE.

[github-badge-img]: https://img.shields.io/badge/Github-7sDream%2Fgurgle-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[github-home]: https://github.com/7sDream/gurgle
[doc-badge-img]: https://img.shields.io/badge/docs-on_docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=read-the-docs
[doc-home]: https://docs.rs/gurgle/latest/gurgle/
