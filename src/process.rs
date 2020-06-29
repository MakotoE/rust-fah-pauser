use super::*;
extern crate test;

#[cfg(windows)] extern crate winapi;
#[cfg(windows)] use winapi::um::handleapi::INVALID_HANDLE_VALUE;
#[cfg(windows)] use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};

#[cfg(target_family = "unix")]
pub fn found_process(process_names: &[String]) -> Result<bool> {
    let output = std::process::Command::new("ps")
        .args(&["-a", "-x", "-o", "command", "--no-headers"])
        .output()?;

    for command in output.stdout.split(|&b| b == b'\n') {
        if let Ok(name) = command_file_name(command) {
            for process_name in process_names {
                if process_name == &name {
                    return Ok(true)
                }
            }
        }
    }

    Ok(false)
}

fn command_file_name(command: &[u8]) -> Result<String> {
    let mut command_parts = command.split(|&b| b == b' ');
    let path = match command_parts.next() {
        Some(mut s) => {
            if s == b"/bin/sh" || s == b"/bin/bash" {
                // Get name of script
                if let Some(script) = command_parts.next() {
                    s = script;
                }
            }

            s
        },
        None => command,
    };

    match std::path::Path::new(std::str::from_utf8(path)?).file_name() {
        Some(name) => match name.to_str() {
            Some(name_str) => Ok(name_str.to_string()),
            None => Err("invalid name".into()),
        },
        None => Ok("".into()),
    }
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

#[cfg(windows)]
fn chars_equal(s: &str, chars: &[winapi::um::winnt::CHAR; 260]) -> bool {
    let mut s_iter = s.bytes();
    for (i, &c) in chars.iter().enumerate() {
        if c == 0 {
            return i == s.len();
        }

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
mod tests {
    use super::*;

    #[test]
    fn test_found_process() {
        process::found_process(Vec::new().as_slice()).unwrap();
    }

    #[test]
    fn test_command_file_name() {
        struct Test {
            command: &'static [u8],
            expected: &'static str,
        }

        let tests: Vec<Test> = vec![
            Test {
                command: b"",
                expected: "",
            },
            Test {
                command: b"a",
                expected: "a",
            },
            Test {
                command: b"/a/b",
                expected: "b",
            },
            Test {
                command: b"/bin/sh a",
                expected: "a",
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            assert_eq!(command_file_name(test.command).unwrap(), test.expected, "{}", i);
        }
    }

    #[test]
    #[cfg(windows)]
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
                chars: &[1],
                expected: false,
            },
            Test {
                s: "a",
                chars: &[],
                expected: false,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let mut chars: [winapi::um::winnt::CHAR; 260] = [0; 260];
            for (i, &c) in test.chars.iter().enumerate() {
                chars[i] = c;
            }
            assert_eq!(chars_equal(test.s, &chars), test.expected, "{}", i);
        }
    }

    #[bench]
    fn bench_found_process(b: &mut test::Bencher) {
        // Windows: test process::tests::bench_found_process ... bench:   2,798,980 ns/iter (+/- 180,655)
        // Linux:   test process::tests::bench_found_process ... bench:   7,768,935 ns/iter (+/- 878,042)
        b.iter(|| found_process(&[]));
    }
}
