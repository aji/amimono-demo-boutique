fn main() {
    let rev = amimono_build::AppDigest::new()
        .add_glob("src/**/*.rs")
        .add_glob("src/**/*.json")
        .add_glob("static/**/*")
        .add_path("Cargo.lock")
        .add_path("Cargo.toml")
        .compute();

    println!("cargo:rustc-env=APP_REVISION={}", rev);
}
