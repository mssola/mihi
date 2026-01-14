A self assessment tool for learning languages.

**UNDER CONSTRUCTION**

## Contributing

This application is built like any other Rust project:

```
$ cargo build
```

In order to test things you need to provide a database for it. One is provided
in the form of `testdata/test.sqlite3`. So take that and set the `MIHI_DATABASE`
environment variable to its full path. And after that you will be able to run:

```
$ cargo test
$ cargo clippy --all-targets --all-features
```

## License

This repository holds two licenses, as you can also note on the `Cargo.toml`
file. As it's written there:

- The source code on the `crates/` directory is licensed under the GNU GPLv3 (or
  any later version).
- The source code on the `lib/` directory is licensed under the GNU LGPLv3 (or
  any later version).

In practice, for the libraries under `lib/` this means that if you plan to
compile your binary statically, you still need to abide by the LGPLv3+ license.
This means at least providing the object files necessary to allow someone to
recompile your program using a modified version of these libraries. See the
LGPLv3 license for more details.
