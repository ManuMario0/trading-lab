fn main() {
    cxx_build::bridge("src/ffi/mod.rs")
        .std("c++14")
        .compile("trading-core");

    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
}
