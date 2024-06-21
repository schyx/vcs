/// Executes `vcs init` with `args` as arguments
///
/// If there are no arguments, executes `vcs init` in the current directory.
/// If there is one argument, executes `vcs init` in the directory named by the argument.
/// In the directory `init` is working in, creates the `.vcs` directory with `HEAD`, `branches`,
///     `objects` subfolders
/// If already in a vcs directory, logs `Already in a vcs directory.` to the console.
///
/// * `args` - arguments `init` was called with
pub fn init(args: Vec<String>) -> String {
    String::from("")
}

#[cfg(test)]
mod tests {
    /*
     * Testing strategy for `init`:
     *      Partition on number of arguments: 0, 1, >1
     *      Partition on whether vcs was already initialized: yes, no
     */

    use super::*;

    #[test]
    fn more_than_one_argument() {
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
        ];
        assert_eq!(
            "Incorrect number of arguments. Expected 0 or 1 arguments.",
            init(test_args)
        );
    }
}
