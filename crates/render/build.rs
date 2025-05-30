use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

fn main() {
    println!("cargo::rerun-if-changed=./shaders");

    let out_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("shaders/");

    if !std::fs::exists(&out_dir).unwrap() {
        std::fs::create_dir(&out_dir).unwrap();
    }

    let mut compilations = vec![];
    for entry in std::fs::read_dir("./shaders").unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let file_path = entry.path();
            let name = PathBuf::from(file_path.file_name().unwrap());
            let out_filepath = out_dir.join(name.with_extension("wgsl"));

            let process = std::process::Command::new("slangc")
                .arg(&file_path)
                .arg("-o")
                .arg(out_filepath)
                .args(["-warnings-as-errors", "all"])
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            compilations.push((name, process));
        }
    }

    for (file, process) in compilations {
        let output = process.wait_with_output().unwrap();
        if !output.status.success() {
            panic!(
                "{}\n{}",
                file.to_string_lossy(),
                String::from_utf8_lossy(&output.stderr)
            )
        }
    }
}
