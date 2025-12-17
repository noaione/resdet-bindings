use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let resdet_dir = manifest_dir.join("resdet");

    // LDLIBS= -lm
    // do not include libm on Windows MSVC
    if !cfg!(target_env = "msvc") {
        println!("cargo:rustc-link-lib=m");
    }
    let out_dir = build_resdet_with_cc(&resdet_dir);

    // Tell cargo to link the static library
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=resdet");

    // Tell cargo to re-run if these files change
    println!("cargo:rerun-if-changed={}/lib", resdet_dir.display());
    println!(
        "cargo:rerun-if-changed={}/include/resdet.h",
        resdet_dir.display()
    );
    println!("cargo:rerun-if-changed=build.rs");
}

fn build_resdet_with_cc(resdet_dir: &Path) -> PathBuf {
    let mut build = cc::Build::new();

    let std_flags = if cfg!(target_env = "msvc") {
        "c11"
    } else {
        "c99"
    };

    // -std=c99 -pedantic -O3 -march=native -mtune=native -Wall -Werror
    // DEFS= -DUSE_BUILTIN_SIGNBIT
    build
        .std(std_flags)
        .warnings(false)
        .define("USE_BUILTIN_SIGNBIT", None);

    let lib_dir = resdet_dir.join("lib");
    let include_dir = resdet_dir.join("include");

    build
        .include(&lib_dir)
        .include(&include_dir)
        .include(lib_dir.join("kissfft"));

    // core files (OBJS=resdet.o image.o methods.o image/y4m.o)
    let core_sources = vec![
        "lib/resdet.c",
        "lib/image.c",
        "lib/methods.c",
        "lib/image/y4m.c",
    ];

    // native image readers (ifndef OMIT_NATIVE_PGM_PFM_READERS)
    let native_readers = vec!["lib/image/pgm.c", "lib/image/pfm.c"];

    // KissFFT fallback (else block of HAVE_FFTW)
    let kiss_fft_sources = vec![
        "lib/transform/kiss_fft.c",
        "lib/kissfft/kiss_fft.c",
        "lib/kissfft/kiss_fftnd.c",
        "lib/kissfft/kiss_fftndr.c",
        "lib/kissfft/kiss_fftr.c",
    ];

    // add all sources to the build
    for src in core_sources
        .into_iter()
        .chain(native_readers)
        .chain(kiss_fft_sources)
    {
        build.file(resdet_dir.join(src));
    }

    build.compile("resdet"); // libresdet.a

    PathBuf::from(env::var("OUT_DIR").unwrap())
}
