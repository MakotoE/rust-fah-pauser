use super::*;

#[cfg(windows)]
extern crate winapi;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};

#[cfg(target_family = "unix")]
pub fn found_process(process_names: &[String]) -> Result<bool> {
    for pid_entry in std::fs::read_dir("/proc")? {
        let mut exe_path = std::ffi::OsString::from("/proc/");
        exe_path.push(pid_entry?.file_name());
        exe_path.push("/exe");

        if let Ok(link_path) = std::fs::read_link(exe_path) {
            for process_name in process_names {
                if let Some(file_name) = link_path.file_name() {
                    if file_name == process_name {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

#[cfg(target_family = "windows")]
pub fn found_process(process_names: &[String]) -> Result<bool> {
    let snapshot_handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot_handle == INVALID_HANDLE_VALUE {
        return Err("CreateToolhelp32Snapshot failed".into());
    }

    let mut process_entry: PROCESSENTRY32 = unsafe { std::mem::zeroed() };
    process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

    if unsafe { Process32First(snapshot_handle, &mut process_entry) } == 0 {
        return Err("Process32First failed".into());
    }

    for process in process_names {
        if chars_equal(process, &process_entry.szExeFile) {
            return Ok(true);
        }
    }

    loop {
        unsafe {
            process_entry.szExeFile = std::mem::zeroed();
        }
        if unsafe { Process32Next(snapshot_handle, &mut process_entry) } == 0 {
            break;
        }

        for process in process_names {
            if chars_equal(process, &process_entry.szExeFile) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn chars_equal(s: &str, chars: &[winapi::um::winnt::CHAR]) -> bool {
    if chars.len() != s.len() {
        return false;
    }

    let mut s_iter = s.bytes();
    for &c in chars {
        if let Some(s_byte) = s_iter.next() {
            if c as u8 != s_byte {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_found_process() {
        process::found_process(Vec::new().as_slice()).unwrap();
    }

    #[test]
    fn test_equal() {
        struct Test {
            s: &'static str,
            chars: &'static [winapi::um::winnt::CHAR],
            expected: bool,
        }

        let tests: Vec<Test> = vec![
            Test {
                s: "",
                chars: &[],
                expected: true,
            },
            Test {
                s: "a",
                chars: &[97],
                expected: true,
            },
            Test {
                s: "",
                chars: &[-1],
                expected: false,
            },
            Test {
                s: "",
                chars: &[0],
                expected: false,
            },
            Test {
                s: "a",
                chars: &[],
                expected: false,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            assert_eq!(chars_equal(test.s, test.chars), test.expected, "{}", i);
        }
    }
}
