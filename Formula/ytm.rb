class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.7"

  depends_on "fzf"
  
  # Note: mpv is required but not auto-installed to avoid massive dependency chain
  # Install manually: brew install mpv

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.7/ytm-macos-arm64.tar.gz"
      sha256 "fcf209b033feea3aa947773823f7b313ce4319715b4e2482628f7dd8f4bddb82"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.7/ytm-macos-x86_64.tar.gz"
      sha256 "c8d4f79cfaba0f564805f8a7aec922ef2d678f4a0534941dd8468cae991ee893"
    end
  end

  def install
    bin.install "ytm"
  end
end
