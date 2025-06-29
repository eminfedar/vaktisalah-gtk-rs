use std::{fs, process::Command};

fn compile_po_files(languages: &str) {
    // Create compiled translations dir:

    for i in languages.split_whitespace() {
        let output_path = format!("po/output/{}/LC_MESSAGES", i);
        fs::create_dir_all(&output_path).unwrap();

        let lang_file = format!("po/{}.po", i);
        if fs::exists(&lang_file).unwrap() {
            // File exists, update contents:
            Command::new("msgfmt")
                .args([
                    &lang_file,
                    "-o",
                    &format!("{}/vaktisalah-gtk-rs.mo", &output_path),
                ])
                .output()
                .unwrap();
        }
    }
}

fn update_po_files(languages: &str) {
    for i in languages.split_whitespace() {
        let lang_file = format!("po/{}.po", i);
        if fs::exists(&lang_file).unwrap() {
            // File exists, update contents:
            Command::new("msgmerge")
                .args(["-o", &lang_file, &lang_file, "po/vaktisalah-gtk-rs.pot"])
                .output()
                .unwrap();
        } else {
            // Create new translation file
            Command::new("cp")
                .args(["po/vaktisalah-gtk-rs.pot", &lang_file])
                .output()
                .unwrap();
        }
    }
}

fn create_pot_file() {
    // Generate translations (using "Python" for .blp files works)
    Command::new("xgettext")
        .args([
            "-o",
            "po/vaktisalah-gtk-rs.pot",
            "-f",
            "po/POTFILES",
            "-L",
            "Python",
        ])
        .output()
        .unwrap();
}

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=ui/MainWindow.blp");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.lock");

    create_pot_file();

    let languages = fs::read("po/LINGUAS").unwrap();
    let languages = String::from_utf8(languages).unwrap();

    update_po_files(&languages);
    compile_po_files(&languages);
}
