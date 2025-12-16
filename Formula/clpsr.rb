class Clpsr < Formula
  desc "Normalizes and merges IPv4 CIDR blocks into the minimal covering set"
  homepage "https://github.com/djessup/clpsr"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/djessup/clpsr/releases/download/v1.0.0/clpsr-macos-x86_64.tar.gz"
      sha256 "a45dc31449750e301905aed3c28baeb0b63dd5211b521617f08887596517c8b5"
    else
      url "https://github.com/djessup/clpsr/releases/download/v1.0.0/clpsr-macos-aarch64.tar.gz"
      sha256 "a5bd1a5ce7d14c26528d74daf81a2c703b48c562e5093b2a24e8331e9feeef4b"
    end
  end

  on_linux do
    url "https://github.com/djessup/clpsr/releases/download/v1.0.0/clpsr-linux-amd64.tar.gz"
    sha256 "48dd84023b8e0ec5f8922f5a90752219dbc33fc8a97767fd2851a38329ff7d71"
  end

  def install
    bin.install "clpsr"
  end

  test do
    assert_match "clpsr", shell_output("#{bin}/clpsr --version")
  end
end

