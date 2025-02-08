# lines_minimal_text_editor_rust

A very minimal text editor.

```
Often in life we need to simply pull open a file and jot a few lines.
We don't have time for anything else.

So: Lines

Your files, created by default, will be in: 
    Documents/lines_editor/TODAYSDATE.txt
    
Lines by day.

```

## filename
Create a named file.
```bash
lines meeting_minutes
```


## linux: for small build, use (for me executible is 1.8mb)
```bash
cargo build --profile release-small 
```


## ~Install
Set an executable file as a keyword in the command line interface (CLI) so that entering that keyword calls the executable:

1. Open the bash shell configuration file in a text editor. The configuration file is usually located at ~/.bashrc or ~/.bash_profile. (use whatever edictor: vim, nano, hx (helix), lapce, etc.)
```bash
hx ~/.bashrc
```
or in some systems it may be called 'b'ash_profile'

2. Add an alias for your executable at the end of the file. Replace your_executable with the name of your executable and /path/to/your_executable with the full path to your executable.
```text
alias your_keyword='/path/to/your_executable'
```
e.g.
```text
alias lines='/home/COMPUTERNAME/lines_editor/lines'
```

3. Save and close the text editor. 
- If you used nano, you can do this by pressing: Ctrl x s (control key, x key, s key)
- If you use Helix(hx), Vim(vi), or lines: 'i' to type, then esc for normal mode, then :wq to write and quit

4. Reload the bash shell configuration file to apply the changes.
```bash
source ~/.bashrc
```
or bash_profile