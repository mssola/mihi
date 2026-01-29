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
