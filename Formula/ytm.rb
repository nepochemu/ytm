class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.1"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.1/ytm-macos-arm64.tar.gz"
      sha256 "2f1db5b63d60f749297e651a5aba16372f02c95f04d923d286506ba4270cd350"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.1/ytm-macos-x86_64.tar.gz"
      sha256 "35938f894f461a50e642548a718ca1a542f363897733ee01f804cb969e479c30"
    end
  end

  def install
    bin.install "ytm"
  end
end
