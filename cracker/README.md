Simple program to generate collisions in a hashing algorithm. 
It was designed to be fast and to be importable from python via the PYo3 crate

## How to use

run `cargo build --release` and then in the `release` folder 

    on macOS, rename libyour_module.dylib to your_module.so.
    on Windows, rename libyour_module.dll to your_module.pyd.
    on Linux, rename libyour_module.so to your_module.so.

run with `python main.py`

### M1 compilation

In order for this code to run under M1 architecture one needs to add to `~/.cargo/config

```
[target.x86_64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]

[target.aarch64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]
```

