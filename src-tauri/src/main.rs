#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Linux: if the app was started from a terminal, start a copy of itself in a new session
/// (no terminal attached) and exit. Closing the terminal then no longer closes the app.
/// Set RM_FOREGROUND=1 to stay in the terminal (for debugging).
#[cfg(target_os = "linux")]
fn detach_from_terminal() {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    if std::env::var_os("RM_DETACHED").is_some() || std::env::var_os("RM_FOREGROUND").is_some() {
        return;
    }
    // SAFETY: isatty only inspects a file descriptor.
    let from_terminal = unsafe { libc::isatty(0) == 1 || libc::isatty(1) == 1 || libc::isatty(2) == 1 };
    if !from_terminal {
        return;
    }
    // For an AppImage, $APPIMAGE is the real file; its temporary mount dies with the terminal.
    let appimage = std::env::var_os("APPIMAGE").map(std::path::PathBuf::from);
    let exe = appimage.clone().or_else(|| std::env::current_exe().ok());
    let Some(exe) = exe else { return };

    let mut cmd = Command::new(exe);
    cmd.args(std::env::args_os().skip(1))
        .env("RM_DETACHED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if appimage.is_some() {
        // Unpack to a temporary folder instead of a FUSE mount: the mount helper would
        // otherwise be a second process that dies with the terminal (and needs fuse2).
        cmd.env("APPIMAGE_EXTRACT_AND_RUN", "1");
    }
    // SAFETY: setsid is async-signal-safe and allowed between fork and exec.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    if cmd.spawn().is_ok() {
        std::process::exit(0);
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    {
        detach_from_terminal();
        // On pure Wayland, apps cannot place windows or keep them "always on top", so the
        // reminder popup would land in the wrong spot. Use X11 (XWayland) when it exists.
        if std::env::var_os("GDK_BACKEND").is_none() && std::env::var_os("DISPLAY").is_some() {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        // Avoids a blank window on some graphics drivers.
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    remember_me_lib::run()
}
