class Clpsr < Formula
  desc "Normalizes and merges IPv4 CIDR blocks into the minimal covering set"
  homepage "https://github.com/djessup/clpsr"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/djessup/clpsr/releases/download/v1.0.0/clpsr-macos-x86_64.tar.gz"
      sha256 "3ddc31c79da352beba3c8b84d99f97f74043a98d84a4a82e400bea04a640494b"
    else
      url "https://github.com/djessup/clpsr/releases/download/v1.0.0/clpsr-macos-aarch64.tar.gz"
      sha256 "baf72923cc744ede12347de24deaad3d4126db1e5c0c9c37d1fbe80cb11b5d83"
    end
  end

  on_linux do
    url "https://github.com/djessup/clpsr/releases/download/v1.0.0/clpsr-linux-amd64.tar.gz"
    sha256 "e519544fb81280e1bd1e3c4e08801a61f58739d9a412ae0518fb26c95fedc4fa"
  end

  def install
    bin.install "clpsr"
  end

  test do
    assert_match "clpsr", shell_output("#{bin}/clpsr --version")
  end
end

