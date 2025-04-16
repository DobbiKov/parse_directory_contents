use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
};

use clap::{arg, command, Parser, ValueHint};

#[derive(Parser)]
#[command(
    version,
    about = "Parse the files in your directory to a clipboard or a file",
    long_about = "Goes through the given diretory and copies the contents of its files to the clipboard OR if an output file is provided then the contents are written to the given file so you can feed it to an LLM"
)]
struct Cli {
    #[arg(value_name = "PATH_TO_DIRECTORY", value_hint=ValueHint::DirPath, help="path to the directory to read")]
    path: PathBuf,

    #[arg(short, long, value_hint=ValueHint::FilePath, help="file to give output to")]
    output_file: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE_EXTENSIONS", help="extensions to read, if not set, the program reads all the files", num_args = 1..)]
    file_types: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();

    let filtered_files = filter_files(read_directory(&cli.path), cli.file_types);
    println!("Files the content will be parsed of:");
    for el in &filtered_files {
        println!("{}", el.display());
    }
    println!();

    match cli.output_file {
        None => {
            println!("Starting copying contents");
            copy_contents_to_clipboard(filtered_files);
            println!("Finished copying contents");
        }
        Some(output_file) => {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&output_file)
                .unwrap_or_else(|_| {
                    panic!(
                        "Couldn't open the file to write to: {}",
                        &output_file.display()
                    )
                });

            println!("Starting copying contents");
            for el in filtered_files {
                let res = write_contents_to_file(&el, &mut file);
                if res {
                    println!("{} - read and written successfully", &el.display());
                }
            }
            println!("Finished copying contents");
        }
    }
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

fn read_directory(path: &PathBuf) -> Vec<PathBuf> {
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
                if entry_path.is_dir() {
                    let temp_res = read_directory(&entry_path);
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
