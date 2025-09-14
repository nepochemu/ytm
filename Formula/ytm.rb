class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.0.0-dev"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.0.0-dev/ytm-macos-arm64.tar.gz"
      sha256 "<sha256_arm64>"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.0.0-dev/ytm-macos-x86_64.tar.gz"
      sha256 "<sha256_x86_64>"
    end
  end

  def install
    bin.install "ytm"
  end
end
