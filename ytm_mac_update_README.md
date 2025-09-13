# Guide: Updating and Packaging macOS Binaries for ytm

This guide explains the full workflow for updating the `ytm` binary on
macOS and publishing it via Homebrew.

------------------------------------------------------------------------

## 1. Get the Latest Source Code

If you already have the repo:

``` bash
cd ~/ytm
git pull origin master
```

If you don't have it yet:

``` bash
git clone https://github.com/nepochemu/ytm.git
cd ytm
```

------------------------------------------------------------------------

## 2. Build a New Release Binary

For Apple Silicon (M1/M2/M3):

``` bash
cargo build --release
tar -czvf ytm-macos-arm64.tar.gz -C target/release ytm
```

For Intel Macs:

``` bash
cargo build --release
tar -czvf ytm-macos-x86_64.tar.gz -C target/release ytm
```

------------------------------------------------------------------------

## 3. Generate Checksums

Run this for each tarball you created:

``` bash
shasum -a 256 ytm-macos-arm64.tar.gz
shasum -a 256 ytm-macos-x86_64.tar.gz
```

Save both SHA256 values.

------------------------------------------------------------------------

## 4. Upload to GitHub Release

1.  Go to [ytm Releases](https://github.com/nepochemu/ytm/releases)
2.  Click **Draft a new release**
3.  Create a new tag, e.g. `v0.1.1`
4.  Upload both tarballs:
    -   `ytm-macos-arm64.tar.gz`
    -   `ytm-macos-x86_64.tar.gz`
5.  Publish release

------------------------------------------------------------------------

## 5. Update the Homebrew Formula

In your `homebrew-ytm` repo, edit `Formula/ytm.rb`:

``` ruby
class Ytm < Formula
  desc "YouTube terminal music client"
  homepage "https://github.com/nepochemu/ytm"
  version "0.1.1"

  on_macos do
    on_arm do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.1/ytm-macos-arm64.tar.gz"
      sha256 "<sha256_arm64>"
    end

    on_intel do
      url "https://github.com/nepochemu/ytm/releases/download/v0.1.1/ytm-macos-x86_64.tar.gz"
      sha256 "<sha256_x86_64>"
    end
  end

  def install
    bin.install "ytm"
  end
end
```

Replace `<sha256_arm64>` and `<sha256_x86_64>` with the values from step
3.

Commit and push:

``` bash
git add Formula/ytm.rb
git commit -m "ytm: update to v0.1.1"
git push
```

------------------------------------------------------------------------

## 6. Test Installation

``` bash
brew uninstall ytm
brew install nepochemu/ytm/ytm
ytm -v
```

You should see:

    ytm 0.1.1

------------------------------------------------------------------------

✅ Done! Your new macOS binary is packaged and available via Homebrew.
