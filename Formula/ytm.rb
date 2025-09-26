class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.3.0"

  depends_on "fzf"
  
  # Note: mpv is required but not auto-installed to avoid massive dependency chain
  # Install manually: brew install mpv

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.3.0/ytm-macos-arm64.tar.gz"
      sha256 "31a10aa448710fb70ceafa5882b27bf0d9e5ee253e72dae0a6e9b7527e33e4f6"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.3.0/ytm-macos-x86_64.tar.gz"
      sha256 "5d6e8534c66d547fecb30668d86b7d6d700f69d4c1fd65fa3027e94713ad5087"
    end
  end

  def install
    bin.install "ytm"
  end
end
