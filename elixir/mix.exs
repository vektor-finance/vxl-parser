defmodule VXLParser.MixProject do
  use Mix.Project

  @version "0.0.2"

  def project do
    [
      app: :vxl_parser,
      version: @version,
      elixir: "~> 1.13",
      description: "VXL Parser for Elixir using Rust NIF",
      deps: deps(),
      package: package(),
      rustler_crates: rustler_crates()
    ]
  end

  defp deps,
    do: [
      {:jason, "~> 1.0"},
      {:rustler, "== 0.22.0"}
    ]

  defp package,
    do: [
      name: "vxl_parser",
      maintainers: ["Vektor <engineering@vektor.finance>"],
      links: %{Github: "https://github.com/vektor-finance/vxl-parser/elixir"},
      files: ["lib", "native", "mix.exs", "README.md"]
    ]

  defp rustler_crates do
    [
      vxl_elixir: [
        path: "elixir",
        mode: rustc_mode(Mix.env())
      ]
    ]
  end

  defp rustc_mode(:prod), do: :release
  defp rustc_mode(_), do: :debug
end
