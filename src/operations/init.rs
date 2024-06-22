use std::{
    env::set_current_dir,
    fs::{create_dir, metadata, File},
    path::Path,
};

/// Executes `vcs init` with `args` as arguments
///
/// If there are no arguments, executes `vcs init` in the current directory.
/// If there is one argument, executes `vcs init` in the directory named by the argument.
/// In the directory `init` is working in, creates the `.vcs` directory with `HEAD`, `branches`,
///     `objects` subfolders
/// If already in a vcs directory, logs `Already in a vcs directory.` to the console.
///
/// * `args` - arguments `init` was called with
pub fn init(args: &Vec<String>) -> String {
    match args.len() {
        2 => {
            if directory_exists(".vcs") {
                return String::from("Already in a vcs directory.");
            }
            create_empty_vcs_dir();
            return String::from("");
        }
        3 => {
            if !directory_exists(&args[2]) {
                let _ = create_dir(&args[2]);
            }

            if directory_exists(".vcs") {
                return String::from("Already in a vcs directory.");
            }

            let _ = set_current_dir(&args[2]);
            create_empty_vcs_dir();
            let _ = set_current_dir("..");
            return String::from("");
        }
        _ => String::from("Incorrect number of arguments. Expected 0 or 1 arguments."),
    }
}

/// Returns true iff `path` is a directory that exists
pub fn directory_exists(path: &str) -> bool {
    let path = Path::new(path);
    metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
}

/// Assuming program is in the correct directory, create an empty `.vcs` directory
fn create_empty_vcs_dir() {
    let _ = create_dir(".vcs");
    let _ = create_dir(".vcs/objects");
    let _ = create_dir(".vcs/branches");
    let _ = File::create(".vcs/HEAD");
}

#[cfg(test)]
mod tests {
    /*
     * Testing strategy for `init`:
     *      Partition on number of arguments: 0, 1, >1
     *      Partition on whether vcs was already initialized: yes, no
     */

    use std::env::set_current_dir;

    use super::super::super::utils::test_dir::make_test_dir;
    use super::*;

    #[test]
    fn more_than_one_argument() {
        let _test_dir = Result::expect(make_test_dir(), "Expected TestDir");
        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "init".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
        ];
        assert_eq!(
            "Incorrect number of arguments. Expected 0 or 1 arguments.",
            init(&test_args)
        );
    }

    #[test]
    fn zero_arguments_not_in_vcs_dir() {
        let _test_dir = Result::expect(make_test_dir(), "Expected TestDir");

        let test_args: Vec<String> = vec!["target/debug/vcs".to_string(), "init".to_string()];

        assert_eq!("", init(&test_args));
        check_empty_vcs_directory_exists();

        assert_eq!("Already in a vcs directory.", init(&test_args));
    }

    #[test]
    fn one_argument_not_in_vcs_dir() {
        let _test_dir = Result::expect(make_test_dir(), "Expected TestDir");

        let test_args: Vec<String> = vec![
            "target/debug/vcs".to_string(),
            "init".to_string(),
            "test_dir".to_string(),
        ];

        assert_eq!("", init(&test_args));
        let _ = set_current_dir("test_dir");
        check_empty_vcs_directory_exists();
    }

    fn check_empty_vcs_directory_exists() {
        assert!(directory_exists(".vcs"));
        assert!(directory_exists(".vcs/branches"));
        assert!(directory_exists(".vcs/objects"));
        assert!(file_exists(".vcs/HEAD"));
    }

    /// Returns true iff `path` is a directory that exists
    fn file_exists(path: &str) -> bool {
        let path = Path::new(path);
        metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
    }
}
