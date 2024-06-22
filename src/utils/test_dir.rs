use std::{
    collections::HashSet,
    env::{current_dir, set_current_dir},
    fs::{read_dir, remove_dir_all, remove_file},
    io::Error,
    path::PathBuf,
};

pub struct TestDir {
    dir_name: PathBuf,          // Make this the directory that TestDir restores
    children: HashSet<PathBuf>, // this is the original children in dir_name
}

pub fn make_test_dir() -> Result<TestDir, Error> {
    let path: PathBuf = Result::expect(current_dir(), "Could not get the current directory");

    let mut children: HashSet<PathBuf> = HashSet::new();

    for entry_result in read_dir(&path)? {
        let entry = entry_result?;
        let entry_path = entry.path();
        children.insert(entry_path);
    }

    Ok(TestDir {
        dir_name: path,
        children,
    })
}

impl Drop for TestDir {
    fn drop(&mut self) {
        Result::expect(
            set_current_dir(&self.dir_name),
            "Could not move to directory",
        );

        let mut paths =
            Result::expect(read_dir(&self.dir_name), "Could not read current directory");

        while let Some(path) = paths.next() {
            let path = Result::expect(path, "Could not get path").path();
            if !&self.children.contains(&path) {
                if path.is_dir() {
                    let _ = remove_dir_all(path);
                } else if path.is_file() {
                    let _ = remove_file(path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn check_remove_file() -> Result<(), Error> {
        let mut paths_before: HashSet<PathBuf> = HashSet::new();

        let cur_dir: PathBuf = current_dir()?;
        for path in read_dir(&cur_dir)? {
            paths_before.insert(path?.path());
        }

        {
            let _test_dir = make_test_dir();
            let _ = File::create("test_file.rs");
        }

        for path in read_dir(&cur_dir)? {
            assert!(paths_before.contains(&path?.path()));
        }

        Ok(())
    }
}
