defmodule VXLParser.MixProject do
  use Mix.Project

  @version "0.0.33"
  @source_url "https://github.com/vektor-finance/vxl-parser"

  def project do
    [
      app: :vxl_parser,
      version: @version,
      description: "VXL Parser for Elixir using Rust NIF",
      elixir: "~> 1.15",
      deps: deps(),
      package: package(),
      test_coverage: [tool: ExCoveralls]
    ]
  end

  defp deps,
    do: [
      {:jason, "~> 1.0"},
      {:rustler_precompiled, "~> 0.7.1"},
      {:rustler, "~> 0.30"},
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:excoveralls, "~> 0.18", only: [:test], runtime: false}
    ]

  defp package,
    do: [
      name: "vxl_parser",
      maintainers: ["Vektor <engineering@vektor.finance>"],
      files: ["lib", "vxl_parser", "mix.exs", "checksum-*.exs", "README.md"],
      links: %{"GitHub" => @source_url}
    ]
end
