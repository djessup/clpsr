class Clpsr < Formula
  desc "Normalizes and merges IPv4 CIDR blocks into the minimal covering set"
  homepage "https://github.com/djessup/clpsr"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/djessup/clpsr/releases/download/<RELEASE_VERSION>/clpsr-macos-x86_64.tar.gz"
      sha256 "<SHA256_FOR_X86_64>"
    else
      url "https://github.com/djessup/clpsr/releases/download/<RELEASE_VERSION>/clpsr-macos-arm64.tar.gz"
      sha256 "<SHA256_FOR_ARM64>"
    end
  end

  on_linux do
    url "https://github.com/djessup/clpsr/releases/download/<RELEASE_VERSION>/clpsr-ubuntu-latest.tar.gz"
    sha256 "<SHA256_FOR_LINUX>"
  end

  def install
    bin.install "clpsr"
  end

  test do
    assert_match "clpsr", shell_output("#{bin}/clpsr --version")
  end
end

