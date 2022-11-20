use std::{
    env,
    path::{Path, PathBuf},
};

use jwalk::WalkDirGeneric;

pub struct FileFinderConfig<'a> {
    // Directory to walk, if `None` defaults to `process.cwd`
    pub dir: Option<&'a str>,
    // Function that takes a &std::Path as parameter and returns boolean
    pub filter: Option<Filter>,
    // See jwalk documentation for specifications on the difference between `custom filter` and `custom skip`
    pub skipped_dirs: Option<Filter>, // TODO add enum for filtering with std::Path and jwalk::DirEntry
    pub err_behavior: ErrorBehavior,
}

#[derive(Debug)]
pub enum ErrorBehavior {
    // In case of Err Result will continue execution
    Ignore,
    // Will panic in case an Err Result is found during execution
    Panic,
    // In case of Err Result will continue execution and will log the error message
    Log,
}

type Filter = fn(&Path) -> bool;

pub fn find_files<'a>(config: FileFinderConfig<'a>) -> Vec<PathBuf> {
    let dir_to_scan = match config.dir {
        Some(dir) => dir.to_string(),
        None => env::current_dir().unwrap().to_str().unwrap().to_string(),
    };

    let walk_dir = WalkDirGeneric::<(i32, bool)>::new(dir_to_scan).process_read_dir(
        move |_, _, _, children| {
            // Apply skip
            children
                .iter_mut()
                .for_each(|dir_entry_result| match dir_entry_result {
                    Ok(dir_entry) => {
                        if let Some(skipped) = config.skipped_dirs {
                            if dir_entry.file_type.is_dir() && skipped(dir_entry.path().as_path()) {
                                dir_entry.read_children_path = None;
                            }
                        }
                    }
                    Err(e) => match &config.err_behavior {
                        ErrorBehavior::Ignore => {}
                        ErrorBehavior::Panic => {
                            panic!("Error during Apply Skip operation {}", e);
                        }
                        ErrorBehavior::Log => {
                            println!("Error during Apply Skip operation {}", e);
                        }
                    },
                });

            // Apply filter
            children.retain(|dir_entry_result| {
                match dir_entry_result {
                    Ok(dir_entry) => {
                        if let Some(filter) = config.filter {
                            if dir_entry.file_type.is_file() {
                                return filter(dir_entry.path().as_path());
                            }
                        }
                    }
                    Err(e) => match &config.err_behavior {
                        ErrorBehavior::Ignore => {}
                        ErrorBehavior::Panic => {
                            panic!("Error during Apply filter operation {}", e);
                        }
                        ErrorBehavior::Log => {
                            println!("Error during Apply filter operation {}", e);
                        }
                    },
                }
                true
            });
        },
    );

    walk_dir
        .into_iter()
        .filter_map(|entry| -> Option<PathBuf> {
            match entry {
                Ok(e) => {
                    if e.file_type.is_file() {
                        return Some(e.path());
                    }
                    None
                }
                Err(_) => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use crate::{find_files, FileFinderConfig};

    #[test]
    fn filter_none_skip_none() {
        env::set_current_dir(Path::new("./test_folder")).ok();
        let found_files = find_files(FileFinderConfig {
            dir: None,
            filter: None,
            skipped_dirs: None,
            err_behavior: crate::ErrorBehavior::Ignore,
        });
        assert_eq!(4, found_files.len());
    }

    #[test]
    fn filter_js_files_skip_none() {
        env::set_current_dir(Path::new("./test_folder")).ok();
        let found_files = find_files(FileFinderConfig {
            dir: None,
            filter: Some(|path| path.ends_with(".js")),
            skipped_dirs: None,
            err_behavior: crate::ErrorBehavior::Ignore,
        });
        assert!(found_files.iter().all(|f| f.ends_with(".js")));
    }

    #[test]
    fn filter_none_skip_directories() {
        env::set_current_dir(Path::new("./test_folder")).ok();
        let found_files = find_files(FileFinderConfig {
            dir: None,
            filter: None,
            skipped_dirs: Some(|path| {
                if path.ends_with("this_is_not_scanned") {
                    return true;
                }
                false
            }),
            err_behavior: crate::ErrorBehavior::Ignore,
        });
        assert!(!found_files
            .iter()
            .any(|f| f.ends_with("not_found_sad_face.js")));
    }
}
