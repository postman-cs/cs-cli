use std::env;
use std::io::IsTerminal;
use std::process::{exit, Command};

/// Check if we're running in a terminal and self-launch if not
pub fn ensure_terminal() {
    // Skip terminal launching if we're running tests or via cargo
    if env::var("CARGO").is_ok() || env::var("RUST_LOG").is_ok() || cfg!(test) {
        return;
    }

    // If we're already in a terminal, continue normally
    if std::io::stdout().is_terminal() {
        return;
    }

    // Get the path to our own executable
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get executable path: {}", e);
            exit(1);
        }
    };

    // Get command line arguments to pass through
    let args: Vec<String> = env::args().skip(1).collect();

    #[cfg(target_os = "macos")]
    {
        launch_in_terminal_macos(&exe_path, &args);
    }

    #[cfg(target_os = "windows")]
    {
        launch_in_powershell_windows(&exe_path, &args);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        eprintln!("This application requires a terminal to run.");
        eprintln!("Please run it from your terminal emulator.");
        exit(1);
    }

    // Exit the current process since we've launched in terminal
    exit(0);
}

#[cfg(target_os = "macos")]
fn launch_in_terminal_macos(exe_path: &std::path::Path, args: &[String]) {
    // Build the command to run in Terminal
    let mut cmd = format!("'{}'", exe_path.display());
    for arg in args {
        cmd.push_str(&format!(" '{}'", arg.replace("'", "'\\''")));
    }

    // AppleScript to open Terminal and run our command
    let apple_script = format!(
        r#"
        tell application "Terminal"
            activate
            do script "{}"
        end tell
        "#,
        cmd
    );

    // Execute the AppleScript
    let result = Command::new("osascript")
        .arg("-e")
        .arg(&apple_script)
        .spawn();

    match result {
        Ok(_) => {
            println!("Launching in Terminal...");
        }
        Err(e) => {
            eprintln!("Failed to launch Terminal: {}", e);
            eprintln!("Please run this application from Terminal manually.");
            exit(1);
        }
    }
}

#[cfg(target_os = "windows")]
fn launch_in_powershell_windows(exe_path: &std::path::Path, args: &[String]) {
    // Build the command for PowerShell
    let mut cmd_args = vec![exe_path.to_string_lossy().to_string()];
    cmd_args.extend(args.iter().cloned());

    // Escape and quote arguments properly for PowerShell
    let ps_command = cmd_args
        .iter()
        .map(|arg| {
            if arg.contains(' ') || arg.contains('$') {
                format!("'{}'", arg.replace("'", "''"))
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    // PowerShell command to keep window open after execution
    let full_command = format!(
        "& {{ {} ; Write-Host ''; Write-Host 'Press Enter to exit...' ; Read-Host }}",
        ps_command
    );

    // Launch PowerShell with our command
    let result = Command::new("powershell.exe")
        .args(&[
            "-NoExit",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &full_command,
        ])
        .spawn();

    match result {
        Ok(_) => {
            println!("Launching in PowerShell...");
        }
        Err(e) => {
            // Try Windows Terminal as fallback
            let wt_result = Command::new("wt.exe")
                .args(&["powershell.exe", "-NoExit", "-Command", &full_command])
                .spawn();

            match wt_result {
                Ok(_) => {
                    println!("Launching in Windows Terminal...");
                }
                Err(_) => {
                    eprintln!("Failed to launch PowerShell: {}", e);
                    eprintln!(
                        "Please run this application from PowerShell or Command Prompt manually."
                    );

                    // Show a message box on Windows
                    show_windows_error_dialog();
                    exit(1);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn show_windows_error_dialog() {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr::null_mut;
        use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK};

        unsafe {
            let title: Vec<u16> = OsStr::new("CS-CLI").encode_wide().chain(Some(0)).collect();
            let message: Vec<u16> =
                OsStr::new("Please run cs-cli from PowerShell or Command Prompt")
                    .encode_wide()
                    .chain(Some(0))
                    .collect();

            MessageBoxW(
                null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }
}
