#![feature(test)]

mod process;

fn main() -> Result<()> {
    let verbose = verbose();
    let pause_on = pause_on()?;

    let mut api = connect()?;

    api.unpause_all()?;

    let mut paused = false;

    loop {
        if process::found_process(pause_on.as_slice())? {
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
    }

    errors {

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
}

fn pause_on() -> Result<Vec<String>> {
    let mut config_path = dirs::home_dir().unwrap();
    config_path.push(".config");
    config_path.push("fah-pauser.yml");

    let file = std::fs::File::open(config_path)?;
    let config: Config = serde_yaml::from_reader(file)?;
    Ok(config.pause_on)
}

fn connect() -> Result<fahapi::API> {
    loop {
        let timeout = std::time::Duration::from_micros(500);
        match fahapi::API::connect_timeout(&fahapi::DEFAULT_ADDR, timeout) {
            Ok(api) => return Ok(api),
            Err(_) => {
                eprintln!("connection error; trying again after a bit");
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        }
    }
}
