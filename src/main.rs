use ignore;
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use clap::{arg, command, Parser, ValueHint};

#[derive(Parser)]
#[command(
    version,
    about = "Parses files from a directory and copies their contents to the clipboard or an output file.",
    long_about = "Recursively reads the contents of files in the specified directory. By default, it copies them to your clipboard. If an output file is specified, it writes all the contents to that file instead—ideal for feeding the text into an LLM."
)]
struct Cli {
    #[arg(
        value_name = "PATH_TO_DIRECTORY",
        value_hint = ValueHint::DirPath,
        help = "Path to the directory whose files should be read."
    )]
    path: PathBuf,

    #[arg(
        short,
        long,
        value_hint = ValueHint::FilePath,
        help = "Optional file to write the output to instead of copying to the clipboard."
    )]
    output_file: Option<PathBuf>,

    #[arg(
        short,
        long,
        value_name = "FILE_EXTENSIONS",
        help = "Space-separated list of file extensions to include (e.g. txt rs md). If not provided, all files are included.",
        num_args = 1..
    )]
    file_types: Option<Vec<String>>,

    #[arg(
        long,
        default_value_t = false,
        action = clap::ArgAction::SetTrue,
        required = false,
        help = "If set, includes files and directories listed in the .gitignore file."
    )]
    disable_gitignore: bool,

    #[arg(
        short,
        long,
        value_name = "EXCLUDE_FILES",
        help = "List of files or directories to exclude from reading.",
        num_args = 1..
    )]
    exclude: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();

    // setting files and directories to ignore
    let excluded: Vec<PathBuf> = get_exluded_files(cli.exclude);

    let files = read_directory_iter(&cli.path, cli.disable_gitignore, excluded);
    let filtered_files = filter_files(files, cli.file_types);

    match cli.output_file {
        None => {
            println!("Starting copying contents");
            copy_contents_to_clipboard(filtered_files);
            println!("Finished copying contents");
        }
        Some(output_file) => {
            println!("Starting copying contents");
            if let Err(e) = copy_all_to_the_file(filtered_files, output_file) {
                panic!(
                    "Couldn't write all the contents to the files because: {}",
                    e
                );
            }
            println!("Finished copying contents");
        }
    }
}

fn copy_all_to_the_file(files: Vec<PathBuf>, output_file_name: PathBuf) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output_file_name)?;

    for el in files {
        let res = write_contents_to_file(&el, &mut file);
        if res {
            println!("{} - read and written successfully", &el.display());
        }
    }
    Ok(())
}

fn copy_contents_to_clipboard(input_files: Vec<PathBuf>) {
    let mut res: String = String::new();
    for el in input_files {
        let contents = match copy_contents_from_file(&el) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "Couldn't copy the contents of the file {} because: {}",
                    &el.display(),
                    e
                );
                continue;
            }
        };

        res.push_str(&format!("```{}\n", &el.display()));
        res.push_str(&contents);
        res.push_str("```\n");
        println!("{} - read successfully", &el.display());
    }

    let mut clipboard = clippers::Clipboard::get();
    match clipboard.write_text(res) {
        Ok(_) => println!("Successfully copied the contents of all the files to clipboard"),
        Err(e) => eprintln!("Couldn't copy the contents to the clipboard because: {}", e),
    }
}

fn copy_contents_from_file(input_path: &PathBuf) -> Result<String, std::io::Error> {
    let mut input_file = match OpenOptions::new().read(true).open(input_path) {
        Err(e) => {
            return Err(e);
        }
        Ok(f) => f,
    };
    let mut contents: String = String::new();
    let _ = match input_file.read_to_string(&mut contents) {
        Err(e) => {
            return Err(e);
        }
        Ok(r) => r,
    };
    Ok(contents)
}

fn write_contents_to_file(input_path: &PathBuf, output_file: &mut std::fs::File) -> bool {
    let contents = match copy_contents_from_file(input_path) {
        Ok(r) => r,
        Err(_) => return false,
    };

    let _ = output_file.write_fmt(format_args!("{}", format!("```{}\n", input_path.display())));
    let _ = output_file.write_fmt(format_args!("{}", contents));
    let _ = output_file.write_fmt(format_args!("``` \n"));
    true
}

fn filter_files(pathes: Vec<PathBuf>, file_extensions: Option<Vec<String>>) -> Vec<PathBuf> {
    if file_extensions.is_none() {
        return pathes;
    }
    let exts = file_extensions.unwrap();
    pathes
        .into_iter()
        .filter(|x| {
            let ext = x.extension();
            match ext {
                None => false,
                Some(v) => exts.contains(&v.to_str().unwrap().to_string()),
            }
        })
        .collect()
}

fn read_directory_iter(
    path: &PathBuf,
    read_from_gitignore: bool,
    exclude: Vec<PathBuf>,
) -> Vec<PathBuf> {
    let mut builder = ignore::WalkBuilder::new(path);

    builder.git_ignore(!read_from_gitignore);

    builder.filter_entry(move |entry: &ignore::DirEntry| {
        let entry_path = entry.path();
        !exclude.iter().any(|ex| entry_path.starts_with(ex))
    });

    let walker = builder.build();
    let res: Vec<PathBuf> = walker
        .filter_map(|r| {
            if let Ok(v) = r {
                let t_path = v.into_path();
                if t_path.is_file() {
                    Some(t_path)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    res
}

fn read_directory(path: &PathBuf, ignore: &Vec<PathBuf>) -> Vec<PathBuf> {
    let dir_res = std::fs::read_dir(path);
    let mut res: Vec<PathBuf> = Vec::new();
    if let Err(e) = dir_res {
        eprintln!("Couldn't read {} due to the error: {}", path.display(), e);
        return vec![];
    }
    let dir_iter = dir_res.unwrap();
    for dir_path in dir_iter {
        match dir_path {
            Err(e) => {
                eprintln!("Couldn't read a dir entry due to the error: {}", e);
                continue;
            }
            Ok(entry) => {
                let entry_path = entry.path();

                if ignore.contains(&entry_path) {
                    continue;
                }

                if entry_path.is_dir() {
                    let temp_res = read_directory(&entry_path, &ignore);
                    for el in temp_res {
                        res.push(el);
                    }
                } else {
                    res.push(entry_path);
                }
            }
        }
    }
    res
}

fn get_paths_from_gitignore() -> Vec<PathBuf> {
    let mut res: Vec<PathBuf> = vec![];
    let mut file = match OpenOptions::new().read(true).open(".gitignore") {
        Err(e) => {
            return res;
        }
        Ok(f) => f,
    };
    let mut contents = String::new();
    if let Err(_) = file.read_to_string(&mut contents) {
        return res;
    }
    if contents.len() == 0 {
        return res;
    }

    if !contents.contains("\n") {
        res.push(PathBuf::from("./").join(contents));
    } else {
        let lines = contents.split("\n");
        for line in lines {
            if line.len() > 0 {
                res.push(PathBuf::from("./").join(line));
            }
        }
    }
    res
}
fn get_exluded_files(paths: Option<Vec<String>>) -> Vec<PathBuf> {
    if paths.is_none() {
        return vec![];
    }
    let paths = paths.unwrap();
    paths
        .into_iter()
        .map(|e| PathBuf::from("./").join(e))
        .collect()
}
