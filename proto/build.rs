fn main() {
    println!("cargo::rerun-if-changed=schema/navi.fbs");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::process::Command::new("flatc")
        .arg("--rust")
        .arg("-o")
        .arg(&out_dir)
        .arg("schema/navi.fbs")
        .status()
        .expect("flatc is not found, run \"nix develop\" or \"direnv allow\"");
}
