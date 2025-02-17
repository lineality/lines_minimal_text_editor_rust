// lines is minimal quick and clean text editor in rust
// cargo build --profile release-small 

/*
lines is minimal quick and clean text editor in rust

cli

This is a small-footprint no-load application.
If a line is not being appendded,
this appliation should not being doing anything with 
the file in question.

1. Only touch files when actively appending a new line
2. Create a temporary backup only during the actual append operation
3. Remove the backup immediately after successful append
4. No persistent locks or ongoing state


Append:
/ 1. Creates temporary backup
/ 2. Appends the line
/ 3. Removes backup if successful
/ 4. Restores from backup if append fails

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
If there is a header.txt file in same directroy as the binary file, 
that file will be appended after the date.

# filename
open a filepath or type a new name
```bash
lines pta_meeting
```
This will create a file with name+date.txt as the filename.

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
/// Determines if a given year is a leap year using the Gregorian calendar rules
/// 
/// # Arguments
/// * `year` - Year to check (CE/AD)
/// 
/// # Returns
/// * `bool` - true if leap year, false if not
/// 
/// # Rules
/// - Year is leap year if divisible by 4
/// - Exception: century years must be divisible by 400
/// - Years divisible by 100 but not 400 are not leap years
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

/// Displays the last n lines of a file to standard output
/// Returns an IO Result to properly handle potential file reading errors
/// 
/// # Arguments
/// * `file_path` - Path to the file to display
/// * `num_lines` - Number of lines to show from end of file
/// 
/// # Returns
/// * `io::Result<()>` - Success or error status of the display operation
/// 
/// # Errors
/// Returns error if:
/// - File cannot be opened
/// - File cannot be read
/// - File content cannot be parsed as valid UTF-8
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
/// Gets the header string for a new file          // <-- Duplicated line
/// Combines timestamp with optional header.txt content  // <-- Duplicated line
fn get_header_text() -> io::Result<String> {
    let timestamp = get_timestamp()?;
    let mut header = format!("# {}", timestamp);

    // Get the executable's directory
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not determine executable directory")
    })?;

    // Check for header.txt in the executable's directory
    let header_path = exe_dir.join("header.txt");
    
    // Also check in the current working directory as fallback
    let current_dir_header = Path::new("header.txt");

    if header_path.exists() {
        let header_content = fs::read_to_string(header_path)?;
        header.push_str("  ");
        header.push_str(&header_content);
    } else if current_dir_header.exists() {
        let header_content = fs::read_to_string(current_dir_header)?;
        header.push_str("  ");
        header.push_str(&header_content);
    }

    Ok(header)
}

/// Appends a single line to the file with temporary backup protection
/// 
/// # Arguments
/// * `file_path` - Path to the file being appended to
/// * `line` - Text line to append
/// 
/// # Behavior
/// 1. Creates temporary backup
/// 2. Appends the line
/// 3. Removes backup if successful
/// 4. Restores from backup if append fails
fn append_line(file_path: &Path, line: &str) -> io::Result<()> {
    // Create temporary backup before modification
    let backup_path = if file_path.exists() {
        let bak_path = file_path.with_extension("bak");
        fs::copy(file_path, &bak_path)?;
        Some(bak_path)
    } else {
        None
    };

    // Attempt to append the line
    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .and_then(|mut file| writeln!(file, "{}", line));

    // Handle the result
    match result {
        Ok(_) => {
            // Success: remove backup if it exists
            if let Some(bak_path) = backup_path {
                fs::remove_file(bak_path)?;
            }
            Ok(())
        }
        Err(e) => {
            // Failure: restore from backup if it exists
            if let Some(bak_path) = backup_path {
                fs::copy(&bak_path, file_path)?;
                fs::remove_file(bak_path)?;
            }
            Err(e)
        }
    }
}

/// Main editing loop for the lines text editor
/// 
/// # Arguments
/// * `file_path` - Path to the file being edited
/// 
/// # Returns
/// * `io::Result<()>` - Success or error status of the editing session
/// 
/// # Behavior
/// 1. Displays file path and basic commands
/// 2. If file doesn't exist, creates it with timestamp header
/// 3. Shows last 10 lines of current file content
/// 4. Enters input loop where user can:
///    - Type text and press enter to append a line
///    - Enter 'q', 'quit', or 'exit' to close editor
/// 5. After each append, displays updated last 10 lines
/// 
/// # Errors
/// Returns error if:
/// - Cannot create/access the file
/// - Cannot read user input
/// - Cannot append to file
/// - Cannot display file contents
/// 
/// # Example
/// ```no_run
/// let path = Path::new("notes.txt");
/// editor_loop(&path)?;
/// ```
fn editor_loop(file_path: &Path) -> io::Result<()> {
    println!("Lines  '(q)uit' | 'exit'\n");
    println!("File: {}", file_path.display());

    let stdin = io::stdin();
    let mut input = String::new();

    // Create file with header if it doesn't exist
    if !file_path.exists() {
        let header = get_header_text()?;
        append_line(file_path, &header)?;
        append_line(file_path, "")?;  // blank line after header
    }

    // Display initial tail of file
    println!("\nCurrent file content (last 10 lines):");
    if let Err(e) = display_file_tail(file_path, 10) {
        eprintln!("Error displaying file: {}", e);
    }

    loop {
        input.clear();
        print!("\n> "); // Add a prompt
        io::stdout().flush()?; // Ensure prompt is displayed
        
        if let Err(e) = stdin.read_line(&mut input) {
            eprintln!("Error reading input: {}", e);
            continue;
        }

        let trimmed = input.trim();
        
        if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
            println!("Exiting editor...");
            break;
        }

        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        
        // Append the line with temporary backup protection
        if let Err(e) = append_line(file_path, trimmed) {
            eprintln!("Error writing to file: {}", e);
            continue;
        }
        
        // Display the tail of the file after append
        println!("\nUpdated file content (last 10 lines):");
        if let Err(e) = display_file_tail(file_path, 10) {
            eprintln!("Error displaying file: {}", e);
        }
    }

    Ok(())
}

// fn editor_loop(file_path: &Path) -> io::Result<()> {
//     println!("Lines  '(q)uit' | 'exit'\n");
//     println!("File: {}", file_path.display());

//     let stdin = io::stdin();
//     let mut input = String::new();

//     // Create file with header if it doesn't exist
//     if !file_path.exists() {
//         let header = get_header_text()?;
//         append_line(file_path, &header)?;
//         append_line(file_path, "")?;  // blank line after header
//     }

//     loop {
//         input.clear();
//         print!("\x1B[2J\x1B[1;1H");
        
//         if let Err(e) = stdin.read_line(&mut input) {
//             eprintln!("Error reading input: {}", e);
//             continue;
//         }

//         let trimmed = input.trim();
        
//         if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
//             println!("Exiting editor...");
//             break;
//         }

//         // Append the line with temporary backup protection
//         if let Err(e) = append_line(file_path, trimmed) {
//             eprintln!("Error writing to file: {}", e);
//             continue;
//         }
        
//         // Display the tail of the file
//         if let Err(e) = display_file_tail(file_path, 10) {
//             eprintln!("Error displaying file: {}", e);
//         }
//     }

//     Ok(())
// }

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

/// Represents supported file managers and their launch commands
#[derive(Debug)]
enum FileManager {
    Nautilus,
    Dolphin,
    Thunar,
    Explorer,
    Finder,
}

impl FileManager {
    /// Converts the FileManager enum to its launch command string
    fn get_command(&self) -> &str {
        match self {
            FileManager::Nautilus => "nautilus",
            FileManager::Dolphin => "dolphin",
            FileManager::Thunar => "thunar",
            FileManager::Explorer => "explorer",
            FileManager::Finder => "open",
        }
    }
}

/// Detects the operating system and returns appropriate default file manager
/// Detects and returns the default file manager for the current operating system
/// 
/// # Returns
/// - `Ok(FileManager)` - Enum variant matching the system's default file manager
/// - `Err(io::Error)` - If operating system is unsupported
/// 
/// # Platform-Specific Behavior
/// ## Linux
/// - GNOME: Returns Nautilus
/// - KDE: Returns Dolphin  
/// - XFCE: Returns Thunar
/// - Default/Unknown: Falls back to Nautilus
/// 
/// ## Windows
/// - Returns Explorer
/// 
/// ## macOS
/// - Returns Finder
/// 
/// # Environment Variables Used
/// - `XDG_CURRENT_DESKTOP`: Used on Linux to detect desktop environment
/// 
/// # Errors
/// Returns error if:
/// - Operating system is not Linux, Windows, or macOS
/// - Unable to determine desktop environment on Linux
/// 
/// # Usage Example
/// ```no_run
/// let file_manager = get_default_file_manager()?;
/// let command = file_manager.get_command();
/// // Use command to open files/directories
/// ```
/// 
fn get_default_file_manager() -> io::Result<FileManager> {
    let os = env::consts::OS;
    match os {
        "linux" => {
            // Check for common Linux desktop environments
            if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
                match desktop.to_uppercase().as_str() {
                    "GNOME" => Ok(FileManager::Nautilus),
                    "KDE" => Ok(FileManager::Dolphin),
                    "XFCE" => Ok(FileManager::Thunar),
                    _ => Ok(FileManager::Nautilus), // Default to Nautilus
                }
            } else {
                Ok(FileManager::Nautilus)
            }
        }
        "windows" => Ok(FileManager::Explorer),
        "macos" => Ok(FileManager::Finder),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("Unsupported operating system: {}", os),
        )),
    }
}

/// Opens the specified directory in the system's file manager
/// 
/// # Arguments
/// * `directory` - Path to the directory to open
/// * `file_manager` - Optional specific file manager to use
/// 
/// # Returns
/// * `io::Result<()>` - Success or error opening the file manager
fn open_in_file_manager(directory: &Path, file_manager: Option<FileManager>) -> io::Result<()> {
    // Ensure the directory exists
    if !directory.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Directory does not exist",
        ));
    }

    let fm = match file_manager {
        Some(fm) => fm,
        None => get_default_file_manager()?,
    };

    // Prepare the command and arguments
    let command = fm.get_command();
    let dir_str = directory.to_string_lossy();

    // Execute the file manager command
    match std::process::Command::new(command)
        .arg(&*dir_str)
        .spawn() {
            Ok(_) => {
                println!("Opened {} in {:?}", dir_str, fm);
                Ok(())
            }
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open file manager: {}", e),
            )),
        }
}

/// Lines - A minimal text editor for quick append-only notes
/// 
/// # Usage
///   lines [FILENAME | COMMAND]
/// 
/// # Commands
///   --files     Open file manager at notes directory
/// 
/// # File Handling
/// - Without arguments: Creates/opens yyyy_mm_dd.txt in ~/Documents/lines_editor/
/// - With filename: Creates/opens filename_yyyy_mm_dd.txt in ~/Documents/lines_editor/
/// - With path: Uses exact path if file exists
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "files" | "--files" => {
                let dir_path = if args.len() > 2 {
                    PathBuf::from(&args[2])
                } else {
                    get_default_filepath(None)?.parent().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::NotFound, "Could not determine parent directory")
                    })?.to_path_buf()
                };
                return open_in_file_manager(&dir_path, None);
            },
            _ => {}
        }
    }

    // Original file editing logic...
    let file_path = if args.len() > 1 {
        let arg_path = PathBuf::from(&args[1]);
        if arg_path.exists() {
            arg_path
        } else {
            get_default_filepath(Some(&args[1]))?
        }
    } else {
        get_default_filepath(None)?
    };

    editor_loop(&file_path)
}