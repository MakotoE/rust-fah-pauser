#![feature(test)]

mod process;

error_chain::quick_main!(run);

fn run() -> Result<()> {
    let verbose = verbose();
    let config = config()?;

    Context {
        api: None,
        verbose,
        pause_on: config.pause_on,
        paused: config.start_paused,
    }.start()
}

struct Context {
    api: Option<fahapi::API>,
    verbose: bool,
    pause_on: Vec<String>,
    paused: bool,
}

impl Context {
    fn start(&mut self) -> Result<()> {
        self.api = Some(connect()?);
        if self.paused {
            self.api.as_mut().unwrap().pause_all()?;
        } else {
            self.api.as_mut().unwrap().unpause_all()?;
        }

        loop {
            match self.monitor_loop() {
                Ok(_) => unreachable!(),
                Err(e) => {
                    eprintln!("{}", e);
                    self.api = Some(connect()?);
                }
            }
        }
    }

    fn monitor_loop(&mut self) -> Result<()> {
        loop {
            if process::found_process(&self.pause_on)? {
                if !self.paused {
                    // Found process; fah is unpaused
                    self.api.as_mut().unwrap().pause_all()?;
                    self.paused = true;
                    if self.verbose {
                        eprintln!("pausing fah")
                    }
                }
            } else if self.paused {
                // No process found; fah is paused
                self.api.as_mut().unwrap().unpause_all()?;
                self.paused = false;
                if self.verbose {
                    eprintln!("unpausing fah")
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
}

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        IO(std::io::Error);
        YAML(serde_yaml::Error);
        FAH(fahapi::Error);
        UTF8(std::str::Utf8Error);
    }
}

fn verbose() -> bool {
    if let Some(flag) = std::env::args().nth(1) {
        if flag == "-v" || flag == "--verbose" {
            return true;
        }
    }
    false
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Config {
    pause_on: Vec<String>,
    #[serde(default)]
    start_paused: bool,
}

fn config() -> Result<Config> {
    let mut config_path = dirs::home_dir().unwrap();
    config_path.push(".config");
    config_path.push("fah-pauser.yml");

    let file = std::fs::File::open(config_path)?;
    Ok(serde_yaml::from_reader(file)?)
}

fn connect() -> Result<fahapi::API> {
    loop {
        let timeout = std::time::Duration::from_secs(1);
        match fahapi::API::connect_timeout(&fahapi::DEFAULT_ADDR, timeout) {
            Ok(api) => return Ok(api),
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }
    }
}
