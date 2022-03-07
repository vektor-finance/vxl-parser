use wasm_bindgen::prelude::*;

#[wasm_bindgen(inspectable)]
#[derive(Debug)]
pub struct BuildInfo {
  git_sha: String,

  build_timestamp: String,
  build_semver: String,

  profile: String,
}

#[wasm_bindgen]
impl BuildInfo {
  #[wasm_bindgen(getter)]
  pub fn git_sha(&self) -> String {
    self.git_sha.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn build_timestamp(&self) -> String {
    self.build_timestamp.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn build_semver(&self) -> String {
    self.build_semver.clone()
  }

  #[wasm_bindgen(getter)]
  pub fn profile(&self) -> String {
    self.profile.clone()
  }
}

#[wasm_bindgen(js_name = "buildInfo")]
pub fn build_info() -> BuildInfo {
  BuildInfo {
    git_sha: env!("VERGEN_GIT_SHA_SHORT").to_string(),

    build_timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
    build_semver: env!("VERGEN_BUILD_SEMVER").to_string(),

    profile: env!("VERGEN_CARGO_PROFILE").to_string(),
  }
}
