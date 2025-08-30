use std::process::{Command, ExitStatus};
fn main() {
    let status = Command::new("./build_frontend.bash")
        .status();
    match status.map(|status| status.success()) {
        Ok(true) => {}
        Ok(false) => {println!("cargo::error=Couldn't compile frontend")}
        Err(e) => {println!("cargo::error=Unable to initialize building for frontend: {:?}", e)}
    }
}