# ytm 🎵

A simple CLI YouTube client in Rust — search and play YouTube audio/videos or playlists directly in your terminal using [mpv](https://mpv.io/) with interactive selection powered by [fzf](https://github.com/junegunn/fzf). The project is ispired by excellent [Yutube-TUI](https://github.com/Siriusmart/youtube-tui) project. The intention here is to have pure gui-less client targeted for music listning inside terminal.

---

## ✨ Features

- 🔍 Search YouTube and interactively pick results with `fzf`
- ▶️ Play videos or entire playlists directly with `mpv`
- 🎧 Audio-only mode (`-n`) for background listening
- 🔑 API key is stored securely and prompted automatically if missing
- ⚡ Built with Rust, packaged via Nix flake

---

## 🔑 API Key Setup

ytm uses the **YouTube Data API v3**.  
You must provide a valid API key from Google Cloud. 

### API key Limitations
Google free plan is limited to 100 searches per day.


### Steps to create an API key

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).  
2. Create or select a project.  
3. Enable the **YouTube Data API v3** for your project:
   - APIs & Services → Library → search "YouTube Data API v3" → Enable.
4. Go to **APIs & Services → Credentials**.  
5. Click **+ Create Credentials → API key**.  
   - Copy the generated key.

### Restrict your API key (recommended)

- Under **Key restrictions**, set:
  - **API restrictions** → restrict to *YouTube Data API v3*.  
  - **Application restrictions** →  
    - If you always use from home/server → restrict by IP (IPv4 or IPv6 `/64` prefix).  
    - Otherwise, just leave API restriction in place.




---

## 📦 Installation

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
brew install nepochemu/ytm/ytm
```

This will download the correct prebuilt binary for your Mac (arm64 for Apple Silicon, x86_64 for Intel).

---

## 🚀 Usage


### First run

The first time you run `ytm`, you will be prompted for an API key.  
You can also set or update it manually:

```bash
ytm --api <YOUR_API_KEY>
```

### Interactive search and play

```bash
ytm portishead third
```

- Opens `fzf` with top 50 results (videos + playlists).
- Select an item → plays immediately in `mpv`.

### Audio-only mode

```bash
ytm -n portishead third
```

- Same as above, but forces `mpv` to run with `--no-video`.


### Playlists

If you select a playlist in `fzf` (shown with `[playlist]`), `ytm` will fetch all its videos and queue them in `mpv`.

Playback is controlled directly inside `mpv`:

- **Next video** → `>` (Shift + `.`)  
- **Previous video** → `<` (Shift + `,`)  
- **Pause / Resume** → `SPACE`  
- **Quit** → `q`

This works the same for both normal videos and playlists, but with playlists you can skip forward/back through the queue.



### Update API key

```bash
ytm --api <YOUR_API_KEY>
```

---

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

### Project structure

- `src/main.rs` → CLI parsing & command dispatch
- `src/api.rs` → YouTube API functions (search, playlists, validation)
- `src/config.rs` → API key storage and handling
- `src/commands.rs` → search + fzf integration, play logic, API key setting

---

## 📜 License

MIT
