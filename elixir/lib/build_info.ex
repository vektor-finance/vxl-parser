defmodule Vektor.Runtime.VXL.BuildInfo do
  @derive Jason.Encoder
  defstruct git_sha: nil,
            build_timestamp: nil,
            build_semver: nil,
            profile: nil

  @type t(git_sha, build_timestamp, build_semver, profile) :: %__MODULE__{
          git_sha: git_sha,
          build_timestamp: build_timestamp,
          build_semver: build_semver,
          profile: profile
        }

  @type t :: %__MODULE__{
          git_sha: String.t(),
          build_timestamp: String.t(),
          build_semver: String.t(),
          profile: String.t()
        }
end
