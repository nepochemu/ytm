class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.6"

  depends_on "fzf"
  
  # Note: mpv is required but not auto-installed to avoid massive dependency chain
  # Install manually: brew install mpv

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.6/ytm-macos-arm64.tar.gz"
      sha256 "789cb672cc4a222c9b750ff7f04ec31ba3005fa8617f06bb9e587319b7be666b"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.6/ytm-macos-x86_64.tar.gz"
      sha256 "4d0c3252260925371f1a4c1f909a1d3219707fee9f7a6694679cc8660c7f9786"
    end
  end

  def install
    bin.install "ytm"
  end
end
