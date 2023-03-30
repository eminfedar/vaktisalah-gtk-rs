use std::{fs::File, process::Command, io::{Write, self}};

// Example custom build script.
fn main() -> io::Result<()> {
    // Tell Cargo that if the given file changes, to rerun this build script.
    let output = Command::new("blueprint-compiler")
        .arg("compile")
        .arg("./ui/MainWindow.blp")
        .output()?;

    let mut file = File::create("./ui/MainWindow.ui")?;
    file.write_all(output.stdout.as_slice())?;

    //println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=ui/MainWindow.blp");

    Ok(())
}
