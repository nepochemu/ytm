class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.0"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.0.0-dev/ytm-macos-arm64.tar.gz"
      sha256 "2fc48a0bf20660ecccc78af57b7b806697b448e54f04d52ec44864203f9066eb"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.0.0-dev/ytm-macos-x86_64.tar.gz"
      sha256 "7a58adbac01071a4806acd1659a630ca7cd968e97ed1e46cc52ecec1c30a0333"
    end
  end

  def install
    bin.install "ytm"
  end
end
