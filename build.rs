use std::process::Command;
fn main() {
    println!("cargo::rerun-if-changed=frontend");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=api-types");
    let status = Command::new("./build_frontend.bash")
        .status();
    match status.map(|status| status.success()) {
        Ok(true) => {}
        Ok(false) => {println!("cargo::error=Couldn't compile frontend")}
        Err(e) => {println!("cargo::error=Unable to initialize building for frontend: {:?}", e)}
    }
}