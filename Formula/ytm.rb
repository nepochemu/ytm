class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.1.6"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.6/ytm-macos-arm64.tar.gz"
      sha256 "e84322fb942cc01dc0545723f382efec2e6107f5701280db093ecfc188a91b5c"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.6/ytm-macos-x86_64.tar.gz"
      sha256 "6ed1b831acfe6545a4b2984dfe89b78b170202b1856b910ceebd5e359df421ed"
    end
  end

  def install
    bin.install "ytm"
  end
end
