use std::io;
use std::path::Path;
use std::process::Command;

/// Launches an application, URL, URI scheme, or file.
pub fn open_target(target: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", target])
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(target)
            .spawn()
            .map(|_| ())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map(|_| ())
    }
}

/// Opens a file in the system default text editor or viewer
pub fn open_in_editor(path: &Path) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Try notepad or start
        Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .map(|_| ())
    }

    #[cfg(not(target_os = "windows"))]
    {
        open_target(&path.to_string_lossy())
    }
}
