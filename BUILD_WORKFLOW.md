# 🚀 Build & Release Workflow for YTM (GitHub input, simple flow)

This guide assumes:

- **ytm** is developed in its own repo (with a `flake.nix`).
- Your **NixOS system flake** consumes `ytm` from **GitHub** (not a local path).
- You want a **simple, reliable** flow with no extra toggles.

---

## TL;DR (Checklist)

1) Edit source → **build with Nix** → fix **cargoHash** if it errors.  
2) Bump **Cargo.toml** version → **commit & push**.  
3) Create **tag** `vX.Y.Z` → **push tag**.  
4) In your **system flake**: `nix flake lock --update-input ytm` → **rebuild** → fix **cargoHash** there if needed.  
5) Verify `ytm -v` shows the new version.

---

## 1) Edit source

Do your changes in the ytm repo.

```bash
# in the ytm repository
nvim src/main.rs
```

---

## 2) Build with Nix (catch cargoHash early)

Build *now* so you surface the cargo hash mismatch immediately and fix it **before** tagging.

```bash
# inside the ytm repo (which has its own flake.nix)
nix build . --print-build-logs
```

- If the build **fails** with a message like:
  
  ```
  got:     sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
  expected: sha256-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=
  ```
  
  **Fix it** by pasting the **`got:`** value into your package’s `cargoHash` field (wherever your `buildRustPackage` lives inside the ytm repo). Then **re-run** the build until it succeeds.

> Tip: you can use `lib.fakeSha256` once to get the real hash printed, then replace it with the printed `got:` value.

When `nix build .` succeeds, your packaging in the **ytm repo** is pinned and reproducible.

(Optional for faster dev cycles: `cargo build --release` is fine too, but the hash issue only appears with Nix builds.)

---

## 3) Bump version **and push**

Update `Cargo.toml` so `ytm -v` reflects the new version. Keep tag and Cargo.toml in sync.

```toml
# Cargo.toml
[package]
name = "ytm"
version = "X.Y.Z"
```

Commit **and push**:

```bash
git add Cargo.toml Cargo.lock
git commit -m "Bump version: X.Y.Z"
git push
```

---

## 4) Tag the release **and push the tag**

```bash
git tag vX.Y.Z
git push --tags
```

Now GitHub has the new release tag. (If you use GitHub Actions for releases, this will trigger it.)

---

## 5) Update your **NixOS system flake** and rebuild

In your **system flake** repo (e.g. `/etc/nixos`):

```bash
cd /etc/nixos
# pull the new ytm tag/commit into flake.lock
nix flake update ytm

# rebuild the system (will use the updated ytm)
sudo nixos-rebuild switch --flake .
```

If you get a **cargoSha256 mismatch** *in the system flake’s ytm package* (e.g., you package ytm there, too):

1. Copy the printed **`got:`** hash.
2. Update the **system flake**’s `cargoHash = "sha256-..."` for ytm.
3. Rebuild again:
   
   ```bash
   sudo nixos-rebuild switch --flake .
   ```
4. (Optional) Commit & push your system flake changes if you track it in Git.

---

## 6) Verify

```bash
ytm -v
# → X.Y.Z
```

If you publish binaries via CI, download and run them to confirm the version as well.

---

## (Optional) Automated binaries via GitHub Actions

Add a workflow at `.github/workflows/release.yml` that builds on tags and uploads the artifacts. This keeps users from compiling locally and gives you nice, versioned releases.

---

## Why this order?

- Building **before** tagging finds the cargo hash issue early, so you don’t cut a tag that won’t reproduce.
- Pushing both the **commit** and the **tag** ensures GitHub + Nix inputs see the same code.
- Updating the **system flake** afterwards brings your machines up to the new, locked version.
