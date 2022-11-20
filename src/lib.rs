use std::{
    env,
    path::{Path, PathBuf},
};

use jwalk::{WalkDirGeneric};

pub struct FileFinderConfig<'a> {
    // Directory to walk, if `None` defaults to `process.cwd`
    pub dir: Option<&'a str>,
    // Function that takes a &std::Path as parameter and returns boolean
    pub filter: Option<Filter>,
    // See jwalk documentation for specifications on the difference between `custom filter` and `custom skip`
    pub skipped_dirs: Option<Filter>,
    pub err_behavior: ErrorBehavior,
}

pub enum ErrorBehavior {
    Ignore,
    Panic,
    Log,
}

type Filter = fn(&Path) -> bool;

pub fn find_files<'a>(config: FileFinderConfig<'a>) -> Vec<PathBuf> {
    let dir_to_scan = match config.dir {
        Some(dir) => dir.to_string(),
        None => env::current_dir().unwrap().to_str().unwrap().to_string(),
    };

    let walk_dir = WalkDirGeneric::<((i32), (bool))>::new(dir_to_scan).process_read_dir(
        move |depth, path, read_dir_state, children| {
            // Apply filter
            children.retain(|dir_entry_result| {
                match dir_entry_result {
                    Ok(dir_entry) => {
                        if let Some(filter) = config.filter {
                            return dir_entry.file_type.is_file() && filter(dir_entry.path().as_path());
                        }
                    }
                    Err(_) => {}
                }
                true
            });

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
                    Err(_) => {}
                });
        },
    );

    walk_dir
        .into_iter()
        .filter_map(|entry| -> Option<PathBuf> {
            match entry {
                Ok(e) => Some(e.path()),
                Err(_) => None,
            }
        })
        .collect()
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
