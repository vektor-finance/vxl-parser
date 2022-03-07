fn main() {
  if version_check::is_min_version("1.49.0").unwrap_or(true) {
    println!("cargo:rustc-cfg=stable_i128");
  }
}
