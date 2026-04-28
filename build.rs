use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let sdk_dir = env::var_os("ND2READSDK_DIR")
        .map(PathBuf::from)
        .or_else(default_sdk_dir)
        .expect("set ND2READSDK_DIR to the Nd2ReadSdk root directory");

    println!("cargo:rerun-if-env-changed=ND2READSDK_DIR");
    println!("cargo:rerun-if-env-changed=ND2READSDK_STATIC");
    println!("cargo:rerun-if-changed={}", sdk_dir.display());

    if cfg!(target_os = "windows") {
        link_windows(&sdk_dir);
    } else if cfg!(target_os = "macos") {
        link_macos(&sdk_dir);
    } else if cfg!(target_os = "linux") {
        link_linux(&sdk_dir);
    } else {
        panic!("unsupported target OS for Nd2ReadSdk");
    }
}

fn link_windows(sdk_dir: &Path) {
    let lib_dir = sdk_dir.join("lib");

    if !lib_dir.join("nd2readsdk-static.lib").exists() {
        panic!(
            "could not find Windows Nd2ReadSdk static libraries under {}",
            sdk_dir.display()
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    for lib in [
        "nd2readsdk-static",
        "limfile-static",
        "tiffxx",
        "tiff",
        "jpeg-static",
        "jbig",
        "lzma",
        "zlib",
        "turbojpeg-static",
    ] {
        println!("cargo:rustc-link-lib=static={lib}");
    }
}

fn link_macos(sdk_dir: &Path) {
    let lib_dir = sdk_dir.join("lib");
    let bin_dir = sdk_dir.join("bin");

    if link_unix_static(&lib_dir) {
        return;
    }

    if require_static() {
        panic!(
            "could not find macOS Nd2ReadSdk static libraries under {}",
            sdk_dir.display()
        );
    }

    if link_shared(&bin_dir, "dylib") || link_shared(&lib_dir, "dylib") {
        return;
    }

    panic!(
        "could not find macOS Nd2ReadSdk libraries under {}",
        sdk_dir.display()
    );
}

fn link_linux(sdk_dir: &Path) {
    let lib_dir = sdk_dir.join("lib");
    let bin_dir = sdk_dir.join("bin");

    if link_unix_static(&lib_dir) {
        return;
    }

    if require_static() {
        panic!(
            "could not find Linux Nd2ReadSdk static libraries under {}",
            sdk_dir.display()
        );
    }

    if link_shared(&bin_dir, "so") || link_shared(&lib_dir, "so") {
        return;
    }

    panic!(
        "could not find Linux Nd2ReadSdk libraries under {}",
        sdk_dir.display()
    );
}

fn link_unix_static(lib_dir: &Path) -> bool {
    if !lib_dir.join("libnd2readsdk-static.a").exists() {
        return false;
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    for lib in [
        "nd2readsdk-static",
        "limfile-static",
        "tiffxx",
        "tiff",
        "jpeg",
        "jbig",
        "lzma",
        "z",
    ] {
        println!("cargo:rustc-link-lib=static={lib}");
    }
    link_cpp_runtime();
    true
}

fn link_shared(dir: &Path, extension: &str) -> bool {
    if !dir
        .join(format!("libnd2readsdk-shared.{extension}"))
        .exists()
    {
        return false;
    }

    println!("cargo:rustc-link-search=native={}", dir.display());
    println!("cargo:rustc-link-lib=dylib=nd2readsdk-shared");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
    link_cpp_runtime();
    true
}

fn default_sdk_dir() -> Option<PathBuf> {
    let home = env::var_os("HOME").map(PathBuf::from)?;
    let docs = home.join("Documents");

    for name in preferred_sdk_dir_names() {
        let candidate = docs.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    find_sdk_dir(&docs)
}

fn find_sdk_dir(root: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(root).ok()?;
    let mut candidates = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("nd2readsdk-"))
                .unwrap_or(false)
        })
        .filter(|path| !require_static() || sdk_dir_name(path).contains("static"))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|path| (sdk_preference_rank(path), sdk_dir_name(path)));
    candidates.into_iter().next()
}

fn preferred_sdk_dir_names() -> &'static [&'static str] {
    if require_static() {
        &["nd2readsdk-static", "nd2readsdk-static-1.7.6.0-Macos-armv8"]
    } else {
        &[
            "nd2readsdk-static",
            "nd2readsdk-static-1.7.6.0-Macos-armv8",
            "nd2readsdk-shared",
            "nd2readsdk-shared-1.7.6.0-Macos-armv8",
        ]
    }
}

fn sdk_preference_rank(path: &Path) -> u8 {
    let name = sdk_dir_name(path);
    if name.contains("static") {
        0
    } else if name.contains("shared") {
        1
    } else {
        2
    }
}

fn sdk_dir_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned()
}

fn require_static() -> bool {
    env::var_os("ND2READSDK_STATIC").is_some()
}

fn link_cpp_runtime() {
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=c++");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
