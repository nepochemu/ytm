class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.1.7"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.0.0-dev/ytm-macos-arm64.tar.gz"
      sha256 "e3d0983dcd0eb4b5f0abe6084ba57cf35a195d3125e883c16aea9ae20733a174"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.0.0-dev/ytm-macos-x86_64.tar.gz"
      sha256 "1edf3a3e78776a69b9d6caab2045f3982c2e235e5c3c8cdde4c9dc13575b9442"
    end
  end

  def install
    bin.install "ytm"
  end
end
