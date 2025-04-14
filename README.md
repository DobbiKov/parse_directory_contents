# Parse directory to a file
This CLI copies the content of the files of the provided folder and writes it to the provided output file in the format:
```
```file_name_1.ext
<contents>
\```

```file_name_2.ext
<contents>
\```
```

Useful if you have a huge project and you want to copy the contents of each file with it's name and give it to a LLM for a prompt.

## Usage:
```
files_to_llm <path_to_dir> <path_to_output_file>
```

Example:
```
files_to_llm . output.txt
```
It will copy the contents of all the files from the current folder to the `output.txt`.

### Optional arguments:
If you want to copy the contents of the specific file types, you can write:
```
files_to_llm <path_to_dir> <path_to_output_file> -f <file_types>
```
Exampe:
```
files_to_llm . output.txt -f rs toml
```
It will copy the contents of the files (that are of the `rs` and `toml` extensions) from the current folder to the `output.txt`.

## Installation
1. Ensure you have cargo installed
2. `git clone https://github.com/DobbiKov/parse_directory_to_file`
3. `cd parse_directory_to_file`
4. `cargo build -r`
5. `sudo cp ./target/release/files_to_llm /usr/bin/`
