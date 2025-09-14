class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.1.5"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.5/ytm-macos-arm64.tar.gz"
      sha256 "25bad49d33ae502099e146e6a0fe70505e2c385c04ba5d7b0aea216546748ca3"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.5/ytm-macos-x86_64.tar.gz"
      sha256 "fc09772b1100231e9ecc15a0cbdeaa6bbe9eb747eeaa11ee59bc7257c827d94c"
    end
  end

  def install
    bin.install "ytm"
  end
end
