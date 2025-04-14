use std::{
    fs::OpenOptions,
    io::{Read, Write},
    ops::Deref,
    path::PathBuf,
};

use clap::{arg, command, Parser, Subcommand, ValueHint};

#[derive(Parser)]
#[command(
    version,
    about,
    long_about = "Reads diretory, its files and writes output to the given file so you can feed it to an LLM"
)]
struct Cli {
    /// Optional name to operate on
    #[arg(value_name = "PATH_TO_DIRECTORY", value_hint=ValueHint::DirPath, help="path to the directory to read")]
    path: PathBuf,

    #[arg(value_hint=ValueHint::FilePath, help="file to give output to")]
    output_file: PathBuf,

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

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&cli.output_file)
        .expect(
            format!(
                "Couldn't open the file to write to: {}",
                &cli.output_file.display()
            )
            .as_str(),
        );

    println!("Starting copying contents");
    for el in filtered_files {
        write_contents_to_file(el, &mut file);
    }
    println!("Finished copying contents");
}

fn write_contents_to_file(input_path: PathBuf, output_file: &mut std::fs::File) {
    let mut input_file = match OpenOptions::new().read(true).open(&input_path) {
        Err(e) => {
            eprintln!(
                "Couldn't open the file {} to read, reason: {}",
                &input_path.display(),
                e
            );
            return;
        }
        Ok(f) => f,
    };
    let mut contents: String = String::new();
    let res = match input_file.read_to_string(&mut contents) {
        Err(e) => {
            eprintln!(
                "Couldn't read the file {} to get contents, because: {}",
                input_path.display(),
                e
            );
            return;
        }
        Ok(r) => r,
    };

    output_file.write_fmt(format_args!("{}", format!("```{}\n", input_path.display())));
    output_file.write_fmt(format_args!("{}", contents));
    output_file.write_fmt(format_args!("``` \n"));
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
    let mut dir_iter = dir_res.unwrap();
    while let Some(dir_path) = dir_iter.next() {
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
