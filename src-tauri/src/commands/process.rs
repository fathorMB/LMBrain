use std::process::Command;

/// Prevent non-interactive helper processes from flashing a console window in
/// the Windows desktop app. Interactive agent sessions use PTYs and do not call
/// this helper.
#[cfg(windows)]
pub fn hide_console(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub fn hide_console(_command: &mut Command) {}
