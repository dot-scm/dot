class Dot < Formula
  desc "A git command proxy CLI tool"
  homepage "https://github.com/username/dot"
  url "https://github.com/username/dot/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/username/dot.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "dot #{version}", shell_output("#{bin}/dot --version")
  end
end
