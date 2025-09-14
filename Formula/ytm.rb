class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.4"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.4/ytm-macos-arm64.tar.gz"
      sha256 "0772d8acf961a2efb06a9eea11456ee88d60e6c710c22a3da2a85df91e7b0d6f"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.4/ytm-macos-x86_64.tar.gz"
      sha256 "73dd680b724fb3ad86fa357b0b3ad7c0c581cd3a9d71a5f093710c069b748523"
    end
  end

  def install
    bin.install "ytm"
  end
end
