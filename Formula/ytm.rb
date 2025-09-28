class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.3.1"

  depends_on "fzf"
  depends_on "mpv"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.3.1/ytm-macos-arm64.tar.gz"
      sha256 "95c96ad62af71c3601c3e567665c476142c4ee7c2418f7777e4bc26449d6d5dc"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.3.1/ytm-macos-x86_64.tar.gz"
      sha256 "6a2c2dd6e70dee9553b11cf1a6d8023a71ae2e7379dd1413ca48ce94a173b994"
    end
  end

  def install
    bin.install "ytm"
  end
end
