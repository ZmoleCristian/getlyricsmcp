class Getlyricsmcp < Formula
  desc "MCP server that finds and fetches song lyrics — no API keys, URL-guessing"
  homepage "https://github.com/ZmoleCristian/getlyricsmcp"
  version "0.1.0"
  license "0BSD"

  on_macos do
    on_arm do
      url "https://github.com/ZmoleCristian/getlyricsmcp/releases/download/v0.1.0/getlyricsmcp-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/ZmoleCristian/getlyricsmcp/releases/download/v0.1.0/getlyricsmcp-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "getlyricsmcp"
    man1.install "man/getlyricsmcp.1" if File.exist?("man/getlyricsmcp.1")
  end

  def caveats
    <<~EOS
      Register the MCP server with Claude Code:
        claude mcp add getlyricsmcp -- getlyricsmcp          # this project
        claude mcp add -s user getlyricsmcp -- getlyricsmcp  # everywhere

      Any other MCP client: stdio transport, command "getlyricsmcp", no arguments.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/getlyricsmcp --version")
  end
end
