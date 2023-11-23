defmodule VXLParser.MixProject do
  use Mix.Project

  @version "0.0.27"

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
      {:rustler, "~> 0.30"},
      {:credo, "~> 1.6", only: [:dev, :test], runtime: false},
      {:excoveralls, "~> 0.13", only: [:test], runtime: false}
    ]

  defp package,
    do: [
      name: "vxl_parser",
      maintainers: ["Vektor <engineering@vektor.finance>"],
      links: %{Github: "https://github.com/vektor-finance/vxl-parser"},
      files: ["lib", "native", "mix.exs", "README.md"]
    ]
end
