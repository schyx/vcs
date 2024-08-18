use std::io::Result;

use chrono::DateTime;

use crate::{
    objects::commit::{
        get_commit_message, get_commit_parent, get_commit_time, get_head_commit,
        INITIAL_COMMIT_HASH,
    },
    utils::fs_utils::directory_exists,
};

/// Executes `vcs log` with `args` as arguments
///
/// Will output each commit that the current HEAD is descended from in reverse chronological order.
/// Each commit will be output in the following format:
///     Commit: <COMMIT HASH>
///     Date: <COMMIT DATE IN DOW, MM, DD, YY, H:M:S, UTC time>
///     <COMMIT MESSAGE>.
///
/// Will log `Not in an initialized vcs directory.` if no vcs dir was found, and will log
/// `Incorrect operands.` if more than 1 argument was supplied. If no commits have been made by the
/// user, will log `Your current branch <BRANCH_NAME> has no commits yet.`.
pub fn log(args: &Vec<String>) -> Result<String> {
    assert!(args[1] == "log");
    if !directory_exists(".vcs") {
        return Ok(String::from("Not in an initialized vcs directory."));
    } else if args.len() != 2 {
        return Ok(String::from("Incorrect operands."));
    } else if get_head_commit()? == INITIAL_COMMIT_HASH {
        let branch = "main";
        return Ok(format!(
            "Your current branch {} has no commits yet.",
            branch
        ));
    }
    let mut output: Vec<String> = vec![];
    let mut current_commit_hash = get_head_commit()?;
    while current_commit_hash != INITIAL_COMMIT_HASH {
        let date = get_commit_time(&current_commit_hash)?;
        let naive_date =
            DateTime::from_timestamp(date, 0).expect("Expected commit time to be parsable.");
        let formatted_time = naive_date.format("%a %b %d %H:%M:%S %Y").to_string();
        let commit_message = get_commit_message(&current_commit_hash)?;
        output.push(format!(
            "Commit: {}\nDate: {}\n{}\n",
            current_commit_hash, formatted_time, commit_message
        ));
        current_commit_hash = get_commit_parent(&current_commit_hash)?.unwrap();
    }
    return Ok(output.join("\n"));
}

#[cfg(test)]
pub mod tests {

    // Partitions for log
    // Partition on error condition:
    //      Not in VCS dir, incorrect number of operands, no commits made yet, no error
    // Further parition on no commits made yet:
    //      no commits on main, no commits on another branch
    // Further partition on no error:
    //      One commit have been made, two or more commits have been made

    use std::{
        fs::File,
        io::{Result, Write},
    };

    use chrono::{Local, Utc};

    use crate::{
        operations::{add::add, commit::commit, init::init, log::log, rm::rm},
        utils::test_dir::make_test_dir,
    };

    #[test]
    fn not_in_vcs_dir() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let test_args: Vec<String> = vec![String::from("target/debug/vcs"), String::from("log")];
        assert_eq!("Not in an initialized vcs directory.", log(&test_args)?);
        Ok(())
    }

    #[test]
    fn incorrect_arg_number() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let test_args: Vec<String> = vec![
            String::from("target/debug/vcs"),
            String::from("log"),
            String::from("test.txt"),
        ];
        assert_eq!("Incorrect operands.", log(&test_args)?);
        Ok(())
    }

    #[test]
    fn no_commits() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let logged_output = log(&vec![String::from("target/debug/vcs"), String::from("log")])?;
        assert_eq!(
            "Your current branch main has no commits yet.",
            logged_output
        );
        Ok(())
    }

    #[test]
    fn no_commits_on_another_branch() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        todo!("Add test after branching!");
    }

    #[test]
    fn one_commit_made() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let _ = File::create("test.txt");
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ]);
        let (_, hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ])?;
        let utc_time = Local::now().with_timezone(&Utc);
        let time = utc_time.format("%a %b %d %H:%M:%S %Y").to_string();
        let logged_output = log(&vec![String::from("target/debug/vcs"), String::from("log")])?;
        assert_eq!(
            format!("Commit: {}\nDate: {}\nAdd test.txt\n", hash, time),
            logged_output
        );
        Ok(())
    }

    #[test]
    fn more_than_one_commit_made() -> Result<()> {
        let _test_dir = make_test_dir()?;
        let mut total_log: Vec<String> = vec![];
        let _ = init(&vec![
            String::from("target/debug/vcs"),
            String::from("init"),
        ]);
        let mut file = File::create("test.txt")?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ]);
        let (_, first_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Add test.txt"),
        ])?;
        let first_utc_time = Local::now().with_timezone(&Utc);
        let first_time = first_utc_time.format("%a %b %d %H:%M:%S %Y").to_string();
        total_log.push(format!(
            "Commit: {}\nDate: {}\nAdd test.txt\n",
            first_hash, first_time
        ));
        file.write_all("hi!".as_bytes())?;
        let _ = add(&vec![
            String::from("target/debug/vcs"),
            String::from("add"),
            String::from("test.txt"),
        ]);
        let (_, second_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Write to test.txt"),
        ])?;
        let second_utc_time = Local::now().with_timezone(&Utc);
        let second_time = second_utc_time.format("%a %b %d %H:%M:%S %Y").to_string();
        total_log.push(format!(
            "Commit: {}\nDate: {}\nWrite to test.txt\n",
            second_hash, second_time
        ));
        let _ = rm(&vec![
            String::from("target/debug/vcs"),
            String::from("rm"),
            String::from("test.txt"),
        ])?;
        let (_, third_hash) = commit(&vec![
            String::from("target/debug/vcs"),
            String::from("commit"),
            String::from("Remove test.txt"),
        ])?;
        let third_utc_time = Local::now().with_timezone(&Utc);
        let third_time = third_utc_time.format("%a %b %d %H:%M:%S %Y").to_string();
        total_log.push(format!(
            "Commit: {}\nDate: {}\nRemove test.txt\n",
            third_hash, third_time
        ));
        let logged_output = log(&vec![String::from("target/debug/vcs"), String::from("log")])?;
        total_log.reverse();
        assert_eq!(total_log.join("\n"), logged_output);
        Ok(())
    }
}
