use std::env;
use std::process::Command;

pub fn ensure_terminal() -> bool {
    // Check if we're already in a terminal
    if atty::is(atty::Stream::Stdout) {
        return true;
    }

    // Get the path to our own executable
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(_) => return false,
    };

    // Use AppleScript to open Terminal and run ourselves
    let apple_script = format!(
        r#"
        tell application "Terminal"
            activate
            do script "{}"
        end tell
        "#,
        exe_path.display()
    );

    // Launch Terminal with our binary
    Command::new("osascript")
        .arg("-e")
        .arg(&apple_script)
        .spawn()
        .is_ok()
}