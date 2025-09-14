class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.1.1"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.1/ytm-macos-arm64.tar.gz"
      sha256 "4dba676103cbd4aef82e3d8c7c3a364a27c86135bbfc4df566b2087c6770f265"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.1/ytm-macos-x86_64.tar.gz"
      sha256 "4931b6fa84f805fdbaf5c2ad9a6962e1c8d2ddaefb3acb70a9bea093a9c78015"
    end
  end

  def install
    bin.install "ytm"
  end
end
