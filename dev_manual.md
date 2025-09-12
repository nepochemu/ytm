# ytm Development & Deployment Workflow

## 📌 What we have achieved
- **ytm project**
  - Rust CLI app (`ytm`) with search/play + API key manager.
  - Own `flake.nix` inside the repo → reproducible package build.
  - Hosted on your private GitHub repo `nepochemu/ytm`.

- **System integration**
  - Your `/etc/nixos/flake.nix` imports `ytm` from GitHub and installs it system-wide.
  - No more symlinks, no more `/home/...` hacks.
  - `ytm` binary available globally in `/nix/store/.../bin/ytm`.

- **Authentication fix**
  - Using `nix flake update ytm` as **your user** (with YubiKey agent).
  - Then `sudo nixos-rebuild switch` as root.
  - Clean, works reliably, no broken pinentry.

- **Sudo config**
  - Preserves `SSH_AUTH_SOCK` and `GPG_TTY` for future-proofing.

---

## 🔄 Proper development workflow

### 1. Edit and test locally
```bash
cd ~/youtube-mpc      # your ytm project repo
nix develop           # enter dev shell with Rust, mpv, etc.
cargo run -- search autechre
```
- Use `cargo run` for fast iteration while coding.  
- Use `nix build .` occasionally to check that your flake builds reproducibly.

### 2. Commit and push to GitHub
```bash
git add .
git commit -m "Implement feature XYZ"
git push
```

### 3. Update system flake lock
As your user (`airflower`):
```bash
cd /etc/nixos
nix flake update ytm
```

### 4. Rebuild system
As root:
```bash
sudo nixos-rebuild switch --flake .
```

### 5. Verify
```bash
which ytm
ytm --help
ytm search autechre
```

(Optional: add `--version` flag in your CLI so you can confirm which build is running.)

---

## ⚡ TL;DR Workflow
1. **Develop:** `cargo run`  
2. **Push:** `git push`  
3. **Update system lock:** `nix flake update ytm`  
4. **Rebuild system:** `sudo nixos-rebuild switch --flake .`  


---

## 🛠 Common Errors & Fixes

### 1. `cargoSha256` mismatch
**Error:**
```
hash mismatch in fixed-output derivation 'cargo-deps-ytm-0.1.0'
```
**Fix:**
- Run `nix build .` once, copy the `got: sha256-...` hash, and paste it into `cargoSha256` in your project `flake.nix`.

---

### 2. SSH / YubiKey issues when running `nix flake update` with sudo
**Symptom:** Hanging prompt or broken PIN entry.  
**Fix:**
- Run `nix flake update ytm` as your user (works with your YubiKey agent).  
- Then rebuild as root with `sudo nixos-rebuild switch --flake .`.  
- Alternatively, switch to HTTPS + GitHub token for private repos.

---

### 3. Wrong binary version after rebuild
**Symptom:** Running `ytm` still gives old version.  
**Fix:**
- Remove old manual installs: `rm ~/.local/bin/ytm`.  
- Verify the binary path: `which ytm` should point to `/nix/store/.../bin/ytm`.  
- Add a `--version` flag in your CLI to confirm the running build.

---

### 4. Makefile / env var expansion issues
**Symptom:** `$SSH_AUTH_SOCK` not passed correctly in `make flake`.  
**Fix:**
- In Makefiles, use `$$SSH_AUTH_SOCK` (double dollar signs).  
- Or skip Make and run directly: `nix flake update ytm` as user.

---

### 5. Dirty Git tree warnings
**Symptom:**
```
warning: Git tree '/etc/nixos' is dirty
```
**Fix:**
- Commit or stash changes before updating.  
- Example:
  ```bash
  sudo git add flake.lock
  sudo git commit -m "Update ytm input"
  ```

