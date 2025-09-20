class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.8"

  depends_on "fzf"
  
  # Note: mpv is required but not auto-installed to avoid massive dependency chain
  # Install manually: brew install mpv

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.8/ytm-macos-arm64.tar.gz"
      sha256 "ab2095b350a563435b32542c267cb151db486780b9370f5efe4b35ec94396d94"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.8/ytm-macos-x86_64.tar.gz"
      sha256 "0dc333733c791f6bbc7679c43c1a204c15e797d04dd554d618fbd7761cbede3e"
    end
  end

  def install
    bin.install "ytm"
  end
end
