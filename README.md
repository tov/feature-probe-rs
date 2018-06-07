# feature-probe-rs: probe for rustc features from `build.rs`

To support multiple versions of Rust, it's often necessary to conditionally
compile parts of our libraries or programs. It's possible to allow users to
specify what features to enable, but detection is better, because users get
all the features that their version of Rust supports. And while we could check
the rustc version, it's better to probe for individual features. That way,
code will work both on nightly, and on stable releases after particular features
stabilize, without changes.

## Usage

It’s [on crates.io](https://crates.io/crates/feature-probe), so you can add

```toml
[dev-dependencies]
feature-probe = "0.1.0"
```


