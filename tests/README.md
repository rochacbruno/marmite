# Integration tests

This folder contains integration tests only

The tests here does not call marmite code directly, but use `process::Command` to
execute `cargo run` or `target/binary` and assert the results. 

The unit tests are located on [src/tests](src/tests)