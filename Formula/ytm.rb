class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.9"

  depends_on "fzf"
  
  # Note: mpv is required but not auto-installed to avoid massive dependency chain
  # Install manually: brew install mpv

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.9/ytm-macos-arm64.tar.gz"
      sha256 "a5200d6f900ab42c1f229009e38d79730359f9d731200849348ab6d91bde8c2a"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.9/ytm-macos-x86_64.tar.gz"
      sha256 "effb229b3434c33ac7fa53975513e7a111e7be13330c7c7c19367903dcd87f29"
    end
  end

  def install
    bin.install "ytm"
  end
end
