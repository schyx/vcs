# VCS (Version Control System)

This is my attempt to implement a git-like version control system in Rust!

## Rules
1. If any operation has incorrect number of arguments, operation will output `Incorrect number of arguments. Expected [number of arguments expected] argument(s).`

2. If no operation with that name exists (yet), will output `No operation with that name exists (yet).`

3. If an operation needs to be in a vcs directory, will output ``Expected operation to be in a `vcs` directory.``

### Supported Operations

1. `init`
2. `add`
3. `commit`
4. `rm`


## Notes
1. Tests should be run in parallel. To do this, run `cargo test -- --test-threads=1`.
