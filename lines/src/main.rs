/*
lines is minimal quick and clean text editor in rust

cli

opens either to a target files as in 

```bash
lines filename.txt
```
or by default makes or opens in append-mode a file in 

home/Documents/lines_editor/yyyy_mm_dd.txt

defaults to default terminal size
shows the bottom N rows of doc (maybe just the result of 
```bash
tail home/Documents/lines_editor/yyyy_mm_dd.txt
```

type and hit enter to
append \n and the new text line to the file

exit or quit or q to close program

# Header
The default header of the file is the date.
If there is a header.txt file in the cwd, that file will be appended after the date.

# filename
open a filepath or type a new name
```bash
lines pta_meeting
```
This will create a file with name+date.txt as the filename.

Note: the file date is odd
*/

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// /// Gets a timestamp string in yyyy_mm_dd format using only standard library
// fn get_timestamp() -> io::Result<String> {
//     let time = std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
//     // Convert seconds to days since epoch and basic date math
//     let days_since_epoch = time.as_secs() / (24 * 60 * 60);
//     let year = 1970 + (days_since_epoch / 365);
//     let day_of_year = days_since_epoch % 365;
    
//     // Very basic month calculation (not accounting for leap years)
//     let month = (day_of_year / 30) + 1;
//     let day = (day_of_year % 30) + 1;
    
//     Ok(format!("{:04}_{:02}_{:02}", year, month, day))
// }

/// Gets a timestamp string in yyyy_mm_dd format using only standard library
fn get_timestamp() -> io::Result<String> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    
    let secs = time.as_secs();
    let days_since_epoch = secs / (24 * 60 * 60);

    // These arrays help us handle different month lengths
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    // Calculate year
    loop {
        let year_length = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < year_length {
            break;
        }
        remaining_days -= year_length;
        year += 1;
    }

    // Calculate month and day
    let mut month = 1;
    for (month_idx, &days) in days_in_month.iter().enumerate() {
        let month_length = if month_idx == 1 && is_leap_year(year) {
            29
        } else {
            days
        };

        if remaining_days < month_length {
            break;
        }
        remaining_days -= month_length;
        month += 1;
    }

    let day = remaining_days + 1;

    Ok(format!("{:04}_{:02}_{:02}", year, month, day))
}

/// Helper function to determine if a year is a leap year
fn is_leap_year(year: u64) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else if year % 400 != 0 {
        false
    } else {
        true
    }
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

/// Gets the header string for a new file
/// Combines timestamp with optional header.txt content
/// 
/// # Returns
/// - `Ok(String)` - Header string containing timestamp and optional header.txt content
/// - `Err(io::Error)` - If there's an error reading header.txt (if it exists)
fn get_header_text() -> io::Result<String> {
    // Get timestamp for the default header
    let timestamp = get_timestamp()?;
    let mut header = format!("# {}", timestamp);

    // Check for header.txt in current working directory
    let header_path = Path::new("header.txt");
    if header_path.exists() {
        let header_content = fs::read_to_string(header_path)?;
        header.push_str("  ");
        header.push_str(&header_content);
    }

    Ok(header)
}

/// Creates a new file with header if it doesn't exist
/// Returns the opened file in append mode
fn create_or_open_file(file_path: &Path) -> io::Result<File> {
    // Check if file exists
    let file_exists = file_path.exists();
    
    // Open file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    // If it's a new file, write the header
    if !file_exists {
        let header = get_header_text()?;
        writeln!(file, "{}\n", header)?;
    }

    Ok(file)
}

/// Main editor loop that handles user input and file operations
/// Provides basic text input functionality and handles exit commands
fn editor_loop(file_path: &Path) -> io::Result<()> {
    
    let mut file = create_or_open_file(file_path)?;

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
        if let Err(e) = writeln!(file, "{}", trimmed) {
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

/// Gets or creates the default file path for the line editor.
/// If a custom filename is provided, appends the date to it.
/// 
/// # Arguments
/// * `custom_name` - Optional custom filename to use as prefix
/// 
/// # Returns
/// - For default: `{home}/Documents/lines_editor/yyyy_mm_dd.txt`
/// - For custom: `{home}/Documents/lines_editor/custom_name_yyyy_mm_dd.txt`
fn get_default_filepath(custom_name: Option<&str>) -> io::Result<PathBuf> {
    // Try to get home directory from environment variables
    let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).map_err(|e| {
        io::Error::new(io::ErrorKind::NotFound, format!("Could not find home directory: {}", e))
    })?;
    
    // Build the base directory path
    let mut base_path = PathBuf::from(home);
    base_path.push("Documents");
    base_path.push("lines_editor");
    
    // Create all directories in the path if they don't exist
    fs::create_dir_all(&base_path)?;
    
    // Get timestamp for filename
    let timestamp = get_timestamp()?;
    
    // Create filename based on whether custom_name is provided
    let filename = match custom_name {
        Some(name) => format!("{}_{}.txt", name, timestamp),
        None => format!("{}.txt", timestamp),
    };
    
    // Join the base path with the filename
    Ok(base_path.join(filename))
}

/// Main function that initializes the editor and handles command-line arguments
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let file_path = if args.len() > 1 {
        // Check if the argument points to an existing file
        let arg_path = PathBuf::from(&args[1]);
        if arg_path.exists() {
            arg_path
        } else {
            // If file doesn't exist, use the argument as a custom filename
            get_default_filepath(Some(&args[1]))?
        }
    } else {
        get_default_filepath(None)?
    };

    editor_loop(&file_path)
}