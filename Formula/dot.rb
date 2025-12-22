class Dot < Formula
  desc "A git command proxy CLI tool"
  homepage "https://github.com/YOUR_USERNAME/dot"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/YOUR_USERNAME/dot/releases/download/v0.1.0/dot-0.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "2d9fe4dc245bc1e1d7f88a52ca1e6d444d706b47ce98a8fec035572ccbe5914a"
    end
    on_intel do
      url "https://github.com/YOUR_USERNAME/dot/releases/download/v0.1.0/dot-0.1.0-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  def install
    bin.install "dot"
  end

  test do
    assert_match "dot #{version}", shell_output("#{bin}/dot --version")
  end
end
