class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.2.5"

  depends_on "mpv"
  depends_on "fzf"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.5/ytm-macos-arm64.tar.gz"
      sha256 "b21d30fdf80c77d1dc66646d4b3455b5245c5fb5cb8e72cf7d77779d284d917c"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.2.5/ytm-macos-x86_64.tar.gz"
      sha256 "17ba6f9688921c984c3fb4bffcb5c0705284a03b5a77cdaa5b936dfadcadf0ed"
    end
  end

  def install
    bin.install "ytm"
  end
end
