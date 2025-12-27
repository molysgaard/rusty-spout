use std::path::{Path, PathBuf};

/// The Spout2 fork that does not include precompiled `dll`s and `lib`s.
const SPOUT_DIR: &str = "Spout2-lean";
const SPOUT_TAG: &str = "2.007.011";

fn main() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    ensure_spout_initted();
    let (spout_build_dir, lib_dir) = build_spout();

    if let Err(e) = std::fs::write(
        repo_root.join("_spout_dll_path"),
        spout_build_dir
            .join("bin/SpoutLibrary.dll")
            .to_str()
            .unwrap(),
    ) {
        println!("cargo:warning={e}");
    }

    let mut cxx_builder = autocxx_build::Builder::new(
        "src/lib.rs",
        &[spout_build_dir.join("include/SpoutLibrary")],
    )
    .build()
    .expect("Failed to generate autocxx bindings. This might be due to autocxx limitations with certain C++ constructs.");
    cxx_builder
        .flag_if_supported("-std=c++14")
        .compile("spoutlib");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lib.rs");

    println!("cargo:rustc-link-lib=SpoutLibrary");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
}

fn ensure_spout_initted() {
    // TODO this might not be the correct path
    if !Path::new(SPOUT_DIR).exists() {
        // TODO explode
        // if env!("CARGO_NET_OFFLINE ") {
        //     panic!("")
        // }

        let status = std::process::Command::new("git")
            .args(["submodule", "update", "--init", SPOUT_TAG])
            .status()
            .unwrap();

        if !status.success() {
            panic!("Unable to init Spout2 submodule");
        }
    }
}

fn build_spout() -> (PathBuf, PathBuf) {
    // Get the Rust build profile to match CMake build type
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let (build_type, profile_name) = if profile == "release" {
        ("Release", "Release")
    } else {
        ("Debug", "Debug")
    };
    
    let dst = cmake::Config::new(SPOUT_DIR)
        .define("SKIP_INSTALL_ALL", "OFF")
        .define("SKIP_INSTALL_HEADERS", "OFF")
        .define("SKIP_INSTALL_LIBRARIES", "OFF")
        .define("SPOUT_BUILD_CMT", "OFF")
        // The only one we want
        .define("SPOUT_BUILD_LIBRARY", "ON")
        .define("SPOUT_BUILD_SPOUTDX", "OFF")
        .define("SPOUT_BUILD_SPOUTDX_EXAMPLES", "OFF")
        // Set CMake build type to match Rust profile
        // For single-config generators (like Makefiles), use CMAKE_BUILD_TYPE
        .define("CMAKE_BUILD_TYPE", build_type)
        // For multi-config generators (Visual Studio), use profile() method
        .profile(profile_name)
        .build();

    (dst.clone(), dst.join("lib"))
}
