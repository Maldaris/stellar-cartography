fn main() {
    // This tells cargo to rerun this build script if the migrations change
    println!("cargo:rerun-if-changed=migrations");
} 