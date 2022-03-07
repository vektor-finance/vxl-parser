use rustler::NifStruct;

#[derive(Debug, NifStruct)]
#[module = "VXL.BuildInfo"]
struct BuildInfo {
  git_sha: String,

  build_timestamp: String,
  build_semver: String,

  profile: String,
}

#[rustler::nif]
fn build_info() -> BuildInfo {
  BuildInfo {
    git_sha: env!("VERGEN_GIT_SHA_SHORT").to_string(),

    build_timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
    build_semver: env!("VERGEN_BUILD_SEMVER").to_string(),

    profile: env!("VERGEN_CARGO_PROFILE").to_string(),
  }
}
