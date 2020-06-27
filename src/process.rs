use super::*;

#[cfg(target_family = "unix")]
pub fn found_process(process_names: &[std::ffi::OsString]) -> Result<bool> {
    for pid_entry in std::fs::read_dir("/proc")? {
        let mut exe_path = std::ffi::OsString::from("/proc/");
        exe_path.push(pid_entry?.file_name());
        exe_path.push("/exe");

        if let Ok(link_path) = std::fs::read_link(exe_path) {
            for process_name in process_names {
                if let Some(file_name) = link_path.file_name() {
                    if file_name == process_name {
                        return Ok(true)
                    }
                }
            }
        }
    }

    Ok(false)
}