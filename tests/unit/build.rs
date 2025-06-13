fn main() {
    // Linker script for the C library (if needed)
    println!("cargo:rustc-link-arg-bins=-Tlinkall.x");
}
