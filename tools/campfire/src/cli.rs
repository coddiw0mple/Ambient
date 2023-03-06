use serde_derive::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
pub struct Cli {
    pub commands: Vec<String>,
}

impl Cli {
    pub fn execute(&self) {
        // Run the commands
        for command in &self.commands {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                        .args(["/C", &command])
                        .output()
                        .expect("failed to execute process")
            } else {
                Command::new("sh")
                        .args(["-c", &command])
                        .output()
                        .expect("failed to execute process")
            };

            println!("status: {}", output.status);
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
}