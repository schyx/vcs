use std::{collections, env, fs, io, path};

pub struct TestDir {
    dir_name: path::PathBuf, // Make this the directory that TestDir restores
    children: collections::HashSet<path::PathBuf>, // this is the original children in dir_name
}

pub fn make_test_dir() -> Result<TestDir, io::Error> {
    let path: path::PathBuf =
        Result::expect(env::current_dir(), "Could not get the current directory");

    let mut children: collections::HashSet<path::PathBuf> = collections::HashSet::new();

    for entry_result in fs::read_dir(&path)? {
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
            env::set_current_dir(&self.dir_name),
            "Could not move to directory",
        );

        let mut paths = Result::expect(
            fs::read_dir(&self.dir_name),
            "Could not read current directory",
        );

        while let Some(path) = paths.next() {
            let path = Result::expect(path, "Could not get path").path();
            if !&self.children.contains(&path) {
                if path.is_dir() {
                    let _ = fs::remove_dir_all(path);
                } else if path.is_file() {
                    let _ = fs::remove_file(path);
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
    fn check_remove_file() -> Result<(), io::Error> {
        let mut paths_before: collections::HashSet<path::PathBuf> = collections::HashSet::new();

        let cur_dir: path::PathBuf = env::current_dir()?;
        for path in fs::read_dir(&cur_dir)? {
            paths_before.insert(path?.path());
        }

        {
            let _test_dir = make_test_dir();
            let _ = File::create("test_file.rs");
        }

        for path in fs::read_dir(&cur_dir)? {
            assert!(paths_before.contains(&path?.path()));
        }

        Ok(())
    }
}
