permissions:
  contents: write

jobs:
  update-formula:
    runs-on: ubuntu-latest
    needs: [build-macos]         # wait for your build job
    steps:
      - uses: actions/checkout@v4

      - name: Switch to default branch
        run: |
          git fetch origin ${{ github.event.repository.default_branch }}
          git checkout ${{ github.event.repository.default_branch }}

      - name: Download release assets
        uses: robinraju/release-downloader@v1.8
        with:
          repository: ${{ github.repository }}
          tag: ${{ github.ref_name }}
          fileName: "ytm-macos-*.tar.gz"

      - name: Calculate checksums
        id: shasum
        run: |
          ARM_SHA=$(shasum -a 256 ytm-macos-arm64.tar.gz | awk '{print $1}')
          INTEL_SHA=$(shasum -a 256 ytm-macos-x86_64.tar.gz | awk '{print $1}')
          echo "arm=$ARM_SHA" >> $GITHUB_OUTPUT
          echo "intel=$INTEL_SHA" >> $GITHUB_OUTPUT

      - name: Update formula
        run: |
          VERSION=${GITHUB_REF_NAME#v}

          sed -i "s/^  version \".*\"/  version \"${VERSION}\"/" Formula/ytm.rb

          sed -i "s|/releases/download/v[0-9.]\+/ytm-macos-arm64.tar.gz|/releases/download/v${VERSION}/ytm-macos-arm64.tar.gz|" Formula/ytm.rb
          sed -i "s|/releases/download/v[0-9.]\+/ytm-macos-x86_64.tar.gz|/releases/download/v${VERSION}/ytm-macos-x86_64.tar.gz|" Formula/ytm.rb

          sed -i "/ytm-macos-arm64.tar.gz/{n;s/sha256 \".*\"/sha256 \"${{ steps.shasum.outputs.arm }}\"/}" Formula/ytm.rb
          sed -i "/ytm-macos-x86_64.tar.gz/{n;s/sha256 \".*\"/sha256 \"${{ steps.shasum.outputs.intel }}\"/}" Formula/ytm.rb

      - name: Commit formula update
        run: |
          git config user.name  "github-actions"
          git config user.email "github-actions@github.com"
          git add Formula/ytm.rb
          git commit -m "Update Homebrew formula for ${{ github.ref_name }}"
          git push
