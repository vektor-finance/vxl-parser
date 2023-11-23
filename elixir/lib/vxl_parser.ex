defmodule VXLParser do
  use Rustler, otp_app: :vxl_parser, crate: :vxl_elixir, path: "vxl_elixir"

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
