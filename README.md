# VCS (Version Control System)

This is my attempt to implement a git-like version control system in Rust!

## Rules
1. If any operation has incorrect number of arguments, operation will output `Incorrect number of arguments. Expected [number of arguments expected] argument(s).`

2. If no operation with that name exists (yet), will output `No operation with that name exists (yet).`

3. If an operation needs to be in a vcs directory, will output ``Expected operation to be in a `vcs` directory.``

### Individual Operation Rules

#### `init`
1. If `init` is called when already in a VCS directory, will output `Already in a vcs directory`.
