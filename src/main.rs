#![feature(test)]

mod process;

error_chain::quick_main!(run);

fn run() -> Result<()> {
    let verbose = verbose();
    let config = config()?;

    let mut first_loop = true;
    loop {
        let mut api = connect()?;
        if first_loop {
            if config.start_paused {
                api.pause_all()?;
            } else {
                api.unpause_all()?;
            }
            first_loop = false;
        }

        match monitor_loop(&mut api, verbose, &config) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

fn monitor_loop(api: &mut fahapi::API, verbose: bool, config: &Config) -> Result<()> {
    let mut paused = config.start_paused;

    loop {
        if process::found_process(&config.pause_on)? {
            if !paused {
                // Found process; fah is unpaused
                api.pause_all()?;
                paused = true;
                if verbose {
                    eprintln!("pausing fah")
                }
            }
        } else if paused {
            // No process found; fah is paused
            api.unpause_all()?;
            paused = false;
            if verbose {
                eprintln!("unpausing fah")
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
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
