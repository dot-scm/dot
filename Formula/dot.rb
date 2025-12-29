class Dot < Formula
  desc "A Git proxy for managing hidden directories with version control"
  homepage "https://github.com/username/dot"
  url "https://github.com/username/dot/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"

  depends_on "rust" => :build
  depends_on "git"

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      Before using dot, you need to:
      1. Set your GitHub token: export GITHUB_TOKEN="your_token"
      2. Configure organizations in ~/.dot/dot.conf
      
      See the README for detailed setup instructions.
    EOS
  end

  test do
    # Test that the binary was installed correctly
    assert_match "dot", shell_output("#{bin}/dot --version")
    
    # Test that help works
    assert_match "Git proxy for managing hidden directories", shell_output("#{bin}/dot --help")
  end
end