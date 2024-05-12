fn use_logger() {
    println!("cargo:rustc-link-search=lib");
    println!("cargo:rustc-link-lib=static=LoaderFFI");
    println!("cargo:rustc-link-lib=static=loader");
}

fn main() {
    use_logger()
}
