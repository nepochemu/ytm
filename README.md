# ytm 🎵

A simple CLI YouTube client in Rust — search and play YouTube videos directly in your terminal using `mpv`.

---

## ✨ Features

- 🔍 Search YouTube for videos from the terminal
- ▶️ Play results directly with [mpv](https://mpv.io/)
- 🗂 Caches last search results for quick playback
- ⚡ Built with Rust, packaged via Nix flake

---

## 🔑 API Key Setup

ytm uses the **YouTube Data API v3**.  
You must provide a valid API key from Google Cloud.

### Steps to create an API key

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).  
2. Create or select a project.  
3. Enable the **YouTube Data API v3** for your project:
   - APIs & Services → Library → search "YouTube Data API v3" → Enable.
4. Go to **APIs & Services → Credentials**.  
5. Click **+ Create Credentials → API key**.  
   - Copy the generated key.

### Restrict your API key (important)

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
sudo apt install build-essential pkg-config libssl-dev clang mpv
```

**Arch:**

```bash
sudo pacman -S base-devel pkgconf openssl mpv
```

Then build and install:

```bash
cargo build --release
cp target/release/ytm ~/.local/bin/
```

---

## 🚀 Usage

### Search

```bash
ytm search autechre
```

→ Lists top 20 results.

### Play

```bash
ytm play 2
```

→ Plays the 2nd result from your last search.

### Force audio-only mode

```bash
ytm -n play 1
```

or

```bash
ytm --no-video play 1
```

---

## 🛠 Development

Enter the dev shell:

```bash
nix develop
```

Build and run quickly:

```bash
cargo run -- search autechre
```

Check reproducible Nix build:

```bash
nix build .
./result/bin/ytm --help
```

---

### Project structure

The project has been refactored into modules for clarity:

src/main.rs → CLI parsing & command dispatch

src/api.rs → YouTube API functions (search, validate_key, etc.)

src/config.rs → API key storage and config handling

src/commands.rs → logic for search, play, api subcommands

This structure makes it easier to extend features like playlists in the future.


## 📜 License

MIT
