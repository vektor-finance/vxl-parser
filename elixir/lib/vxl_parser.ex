defmodule VXLParser do
  version = Mix.Project.config()[:version]

  use RustlerPrecompiled,
    otp_app: :vxl_parser,
    crate: "vxl_elixir",
    base_url: "https://github.com/vektor-finance/vxl-parser/releases/download/v#{version}",
    force_build: System.get_env("RUSTLER_PRECOMPILATION_EXAMPLE_BUILD") in ["1", "true"],
    targets: Enum.uniq(["aarch64-unknown-linux-musl" | VXLParser.Config.default_targets()]),
    version: version

  alias BuildInfo

  @doc """
  Parses input to VXL AST (json)
  """
  @spec parse(String.t()) :: {:ok, String.t()} | {:error, :parse_error}
  def parse(_input), do: error()

  @spec build_info() :: BuildInfo.t()
  def build_info, do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
