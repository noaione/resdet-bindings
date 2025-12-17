use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let resdet_dir = manifest_dir.join("resdet");

    let (_, lib_dir) = build_resdet(&resdet_dir);

    // Tell cargo to link the static library
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=resdet");

    // Tell cargo to re-run if these files change
    println!("cargo:rerun-if-changed={}/lib", resdet_dir.display());
    println!(
        "cargo:rerun-if-changed={}/include/resdet.h",
        resdet_dir.display()
    );
    println!("cargo:rerun-if-changed=build.rs");
}

fn build_resdet(resdet_dir: &Path) -> (PathBuf, PathBuf) {
    // build the resdet into rust OUT dir
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let output_dir = out_dir.join("resdet");
    let include_dir = output_dir.join("include");
    let lib_dir = output_dir.join("lib");

    // make dir
    std::fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    std::fs::create_dir_all(&include_dir).expect("Failed to create bin directory");
    std::fs::create_dir_all(&lib_dir).expect("Failed to create lib directory");

    // Run configure with --disable-everything to only build the core library
    let configure_status = std::process::Command::new("sh")
        .arg("configure")
        .arg("--disable-everything")
        .current_dir(&resdet_dir)
        .arg(format!("--prefix={}", output_dir.display()))
        .status()
        .expect("Failed to run configure");

    if !configure_status.success() {
        panic!("configure failed");
    }

    // clean -> build -> install-lib
    let make_clean_status = std::process::Command::new("make")
        .arg("clean")
        .current_dir(&resdet_dir)
        .status()
        .expect("Failed to run make clean");
    if !make_clean_status.success() {
        panic!("make clean failed");
    }
    let make_build_status = std::process::Command::new("make")
        .arg("lib/libresdet.a")
        .current_dir(&resdet_dir)
        .status()
        .expect("Failed to run make");
    if !make_build_status.success() {
        panic!("make failed");
    }
    let make_install_status = std::process::Command::new("make")
        .arg("install-lib")
        .current_dir(&resdet_dir)
        .status()
        .expect("Failed to run make install");
    if !make_install_status.success() {
        panic!("make install failed");
    }

    (include_dir, lib_dir)
}
