use rustc_version::{version_meta, Channel};

fn main() {
    // Set cfg flags depending on release channel. We use "unstable" cfg to gate
    // some of the unstable features in the compiler.
    if version_meta().unwrap().channel != Channel::Stable {
        println!("cargo:rustc-cfg=unstable");
    }
}
