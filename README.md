# ytm - youtube music

A simple CLI YouTube client in Rust — search and play YouTube audio/videos or playlists directly in your terminal using [mpv](https://mpv.io/) with interactive selection powered by [fzf](https://github.com/junegunn/fzf). The project is ispired by excellent [Yutube-TUI](https://github.com/Siriusmart/youtube-tui) project. Motivation for YTM is to have simple head-less client without graphical ui.

---

## Features

- Search YouTube and interactively pick results with `fzf`
- Play videos or entire playlists directly with `mpv`
- Default mode is Audio-only. Video mode (-v) is optional.
- Background listening mode (-b) doesnt occupy active shell.
- Built with Rust, with great help from [Amp](https://github.com/ampcode-com), and [codex](https://chatgpt.com/codex),  packaged via Nix flake



---

## Installation

### NixOS (system-wide)

Add `ytm` as an input in your system flake (`/etc/nixos/flake.nix`):

```nix
ytm.url = "github:nepochemu/ytm";
```

Then update and rebuild:

```bash
nix flake update ytm
sudo nixos-rebuild switch --flake /etc/nixos
```

The binary will be available globally as `ytm`.

---

### Non-Nix systems simple install

Download and extract the prebuilt binary:

```bash
curl -L -o ytm-linux-x86_64.tar.gz https://github.com/nepochemu/ytm/releases/download/v0.2.8/ytm-linux-x86_64.tar.gz
tar -xvzf ytm-linux-x86_64.tar.gz

sudo mv ytm /usr/local/bin/

ytm --version
```


### Non-Nix systems (manual build)

Install dependencies:

**Debian/Ubuntu:**

```bash
sudo apt install build-essential pkg-config libssl-dev clang mpv fzf
```

**Arch:**

```bash
sudo pacman -S base-devel pkgconf openssl mpv fzf
```

Then build and install:

```bash
cargo build --release
cp target/release/ytm ~/.local/bin/
```


**macOS (Homebrew)**

You can install **ytm** on macOS using [Homebrew](https://brew.sh/):

```bash
brew tap nepochemu/ytm https://github.com/nepochemu/ytm
brew install ytm
```

This will download the correct prebuilt binary for your Mac (arm64 for Apple Silicon, x86_64 for Intel).


---

## Usage



### Interactive search and play

```bash
ytm portishead third
```

- Opens `fzf` with top 50 results (videos + playlists).
- Select an item → plays immediately in `mpv`.

### Video mode

```bash
ytm -v portishead third
```

- Same as above, but forces `mpv` to open video window.

### Background playback mode

```bash
ytm -b portishead third
```

- Starts playback in background and returns to terminal
- Shows track information and playlist position  
- Control with: `ytm next`, `ytm prev`, `ytm pause`, `ytm resume`, `ytm stop`, `ytm status`
- Can be combined with `-v` flag: `ytm -b -v portishead third`

### Playlists

If you select a playlist in `fzf` (shown with `[playlist]`), `ytm` will fetch all its videos and queue them in `mpv`.

Playback is controlled directly inside `mpv`:

- **Next video** → `>` (Shift + `.`)  
- **Previous video** → `<` (Shift + `,`)  
- **Pause / Resume** → `SPACE`  
- **Quit** → `q`

This works the same for both normal videos and playlists, but with playlists you can skip forward/back through the queue.




## 🛠 Development

Enter the dev shell:

```bash
nix develop
```

Build and run quickly:

```bash
cargo run -- chlär
```

Check reproducible Nix build:

```bash
nix build .
./result/bin/ytm --help
```

---



## 📜 License

MIT
