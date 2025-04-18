# Parse directory to a file
The CLI goes though the given directory and copies the contents of the found files:
- to the clipboard
- or to the output file if a such is provided

The copied format is:

```
```file_name_1.ext
<contents>
\```

```file_name_2.ext
<contents>
\```
```

Useful if you have a huge project and you want to copy the contents of each file with it's name and give it to a LLM for a prompt.

By default, all the folders and files written in the .gitignore won't be parsed and copied. However, you can specify using `--disable-gitignore` flag in order to copy them as well.

It's possible to specify files and folders that should be excluded (i.e not to be parsed) using `-e` flag and provide files and folders to exclude.

## Usage:
```
files_to_llm <path_to_dir> 

Copies the contents of each file in the directory to the clipboard
```

Example:

```
files_to_llm . 
```
It will copy the contents of all the files from the current folder to the clipboard.

### Optional arguments:
#### If you want to copy the contents of the specific file types:
```
files_to_llm <path_to_dir> -f <file_types>
```
Example:
```
files_to_llm . -f rs toml
```
It will copy the contents of the files (that are of the `rs` and `toml` extensions) from the current folder to the clipboard.

#### If you want to copy the contents to an output file:
```
files_to_llm <path_to_dir> -o <output_file_name>
```

Example:
```
files_to_llm . -o output.txt
```
It will copy the contents of each file in the current directory to the `output.txt` file (attention: it won't copy to the clipboard!).

#### Combine with specific file types:
```
files_to_llm <path_to_dir> -o <output_file_name> -f <file_types>
```

Example:
```
files_to_llm . -o output.txt -f rs toml
```

It will copy the contents of each file  that is of type `.rs` or `.toml` in the current directory to the `output.txt` file (attention: it won't copy to the clipboard!).

## Installation
1. Ensure you have cargo installed
2. `git clone https://github.com/DobbiKov/parse_directory_to_file`
3. `cd parse_directory_to_file`
4. `cargo build -r`
5. `sudo cp ./target/release/files_to_llm /usr/bin/` or `sudo cp ./target/release/files_to_llm /usr/local/bin/` 
