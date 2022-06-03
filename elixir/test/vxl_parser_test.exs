defmodule VXLParserTest do
  use ExUnit.Case

  test "parses function" do
    assert VXLParser.parse("function.subfunction(1, false, \"hello\")") ==
             {:ok,
              "[{\"offset\":0,\"line\":1,\"column\":1,\"token\":{\"function\":{\"name\":{\"offset\":0,\"line\":1,\"column\":1,\"token\":{\"identifier\":\"function\"}},\"subfunction\":{\"offset\":9,\"line\":1,\"column\":10,\"token\":{\"identifier\":\"subfunction\"}},\"args\":[{\"offset\":21,\"line\":1,\"column\":22,\"token\":{\"number\":{\"int\":\"1\"}}},{\"offset\":24,\"line\":1,\"column\":25,\"token\":{\"boolean\":false}},{\"offset\":32,\"line\":1,\"column\":33,\"token\":{\"string\":\"hello\"}}]}}}]"}
  end

  test "gets build info" do
    assert %{
             __struct__: VXL.BuildInfo,
             build_semver: _,
             build_timestamp: _,
             git_sha: _,
             profile: _
           } = VXLParser.build_info()
  end
end
