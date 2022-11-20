# An easy to use library to find the files you want

Just import and call the function, use std lib `&Path` to filter and find your files.

### Usage

``` rs
let config = FileFinderConfig {
    dir: Some("subpath") // Defaults ".",
    filter: Some(|path| {
        path.ends_with(".js") // Gets only js files
    }),
    skipped_dirs: Some(|path| {
        path.ends_with("node_modules") // Skips children of every `node_modules` dir inside `dir` proeprty.
    }),
    err_behavior: ErrorBehavior::Log // Will log error messages but continue execution
}
let found_files: Vec<PathBuf> = find_files(config)
```