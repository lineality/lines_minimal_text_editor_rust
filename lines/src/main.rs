/*
lines is minimal quick and clean text editor in rust

cli

opens either to a target files as in 

```bash
lines filename.txt
```

or by default makes or opens in append-mode a file in 

home/Documents/line_editor/yyyy_mm_dd


defaults to default terminal size
shows the bottom N rows of doc (maybe just the result of 
```bash
tail home/Documents/line_editor/yyyy_mm_dd
```

type and hit enter to
append \n\n and the new text line to the file

exit or quit or q to close program

no unwrap etc.
*/

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Gets a timestamp string in yyyy_mm_dd format using only standard library
fn get_timestamp() -> io::Result<String> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    // Convert seconds to days since epoch and basic date math
    let days_since_epoch = time.as_secs() / (24 * 60 * 60);
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    
    // Very basic month calculation (not accounting for leap years)
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;
    
    Ok(format!("{:04}_{:02}_{:02}", year, month, day))
}

/// Gets the default file path for the line editor
/// Creates a format like: home/Documents/line_editor/yyyy_mm_dd
fn get_default_filepath() -> io::Result<PathBuf> {
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).map_err(|e| {
        io::Error::new(io::ErrorKind::NotFound, format!("Could not find home directory: {}", e))
    })?;
    
    let mut base_path = PathBuf::from(home);
    base_path.push("Documents");
    base_path.push("line_editor");
    
    fs::create_dir_all(&base_path)?;
    
    let filename = get_timestamp()?;
    Ok(base_path.join(filename))
}

/// Displays the last n lines of the file
/// Returns an IO Result to properly handle potential file reading errors
fn display_file_tail(file_path: &Path, num_lines: usize) -> io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<io::Result<_>>()?;
    
    let start = if lines.len() > num_lines {
        lines.len() - num_lines
    } else {
        0
    };

    for line in &lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

/// Main editor loop that handles user input and file operations
/// Provides basic text input functionality and handles exit commands
fn editor_loop(file_path: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    // clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!("Lines  '(q)uit' | 'exit'\n");
    println!("File: {}", file_path.display());

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("\x1B[2J\x1B[1;1H");
        if let Err(e) = stdin.read_line(&mut input) {
            eprintln!("Error reading input: {}", e);
            continue;
        }

        let trimmed = input.trim();
        
        if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
            println!("Exiting editor...");
            break;
        }

        // Add two newlines and the input text
        if let Err(e) = writeln!(file, "\n{}", trimmed) {
            eprintln!("Error writing to file: {}", e);
            continue;
        }
        
        // Display the tail of the file
        if let Err(e) = display_file_tail(file_path, 10) {
            eprintln!("Error displaying file: {}", e);
        }
    }

    Ok(())
}

/// Main function that initializes the editor and handles command-line arguments
/// Returns an IO Result to properly handle potential errors
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let file_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        get_default_filepath()?
    };

    editor_loop(&file_path)
}
