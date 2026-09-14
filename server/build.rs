fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if target.contains("msvc") {
        // Link dynamically against the MSVC C++ runtime library and Universal C Runtime library
        println!("cargo:rustc-link-lib=msvcprt");
        println!("cargo:rustc-link-lib=ucrt");
    }
}
