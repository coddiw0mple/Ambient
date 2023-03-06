use serde_derive::Deserialize;
use std::fs;
use std::process::{exit, Command};
use toml;


#[derive(Deserialize)]
struct Config {
    cli: Cli,
    snr: Snr,
    version: String,
}

#[derive(Deserialize)]
struct Cli {
    commands: Vec<String>,
}

#[derive(Deserialize)]
struct Snr {
    paths: Vec<String>,
}



fn main() {
    let filename = "campfire.toml";

    // Read the contents of the file using a `match` block
    // to return the `data: Ok(c)` as a `String`
    // or handle any `errors: Err(_)`.
    let contents = match fs::read_to_string(filename) {
        // If successful return the files text as `contents`.
        // `c` is a local variable.
        Ok(c) => c,
        // Handle the `error` case.
        Err(_) => {
            // Write `msg` to `stderr`.
            eprintln!("Could not read file `{}`", filename);
            // Exit the program with exit code `1`.
            exit(1);
        }
    };

    let data: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Unable to load data from `{}`", filename);
            eprintln!("Error: {}", err);
            exit(1);
        }
    };

    // Print out the values to `stdout`.
    println!("{:?}", data.cli.commands); // ["ls", "cd", "pwd"]
    println!("{:?}", data.snr.paths); // ["C:\\Users\\user\\Documents\\", "C:\\Users\\user\\Desktop\\"]
    println!("{}", data.version); // "0.1.0"

    // Run the commands
    for command in data.cli.commands {
        let output = Command::new("cmd")
            .args(&["/C", &command])
            .output()
            .expect("failed to execute process");

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

}
