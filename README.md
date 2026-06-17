# ratatui-todo-list

Terminal todo-list application built with [ratatui](https://github.com/ratatui/ratatui).

This repository is an experimental **monorepo** for packaging Rust CLI/TUI tools into a shared APT repository. The application source and Debian packaging live under `base/ratatui-todo-list/`.

## Repository layout

```text
.
├── .github/workflows/apt-repo.yml   # CI/CD for the shared APT repository
├── README.md
└── base/
    └── ratatui-todo-list/           # application source and Debian packaging
        ├── Cargo.toml
        ├── src/
        ├── debian/
        └── Dockerfile
```

## Install from APT repository

### 1. Add the signing key

```bash
sudo wget -qO /usr/share/keyrings/ratatui-todo-list.gpg \
  https://Artem4590.github.io/ratatui-todo-list/KEY.gpg
```

### 2. Add the repository

```bash
echo "deb [signed-by=/usr/share/keyrings/ratatui-todo-list.gpg] \
  https://Artem4590.github.io/ratatui-todo-list stable main" | \
  sudo tee /etc/apt/sources.list.d/ratatui-todo-list.list
```

### 3. Install

```bash
sudo apt update
sudo apt install ratatui-todo-list
```

## Build from source

```bash
cd base/ratatui-todo-list
cargo build --release
./target/release/ratatui-todo-list
```

## Build the Debian package locally

```bash
cd base/ratatui-todo-list
docker build -t ratatui-todo-list-deb .
```

## Maintainer setup

### Enable GitHub Pages

1. Go to **Settings → Pages** in the repository.
2. Select **Deploy from a branch** and choose `gh-pages`.
3. Save.

### Create the `gh-pages` branch

```bash
git checkout --orphan gh-pages
git rm -rf .
git commit --allow-empty -m "Initialize gh-pages"
git push origin gh-pages
```

### Generate a GPG signing key

1. Generate a new RSA signing key (no expiration is easiest for CI, but you can set one):

   ```bash
   gpg --full-generate-key
   ```

   Choose:
   - kind: `(4) RSA (sign only)`
   - keysize: `4096`
   - expiration: `0` (or your preferred value)
   - name/email: any value, e.g. `ratatui-todo-list APT signing`
   - passphrase: **leave empty** for CI, or set one and use `APT_GPG_PASSPHRASE` below

2. Find the key ID:

   ```bash
   gpg --list-secret-keys --keyid-format long
   ```

   Look for a line like:

   ```text
   sec   rsa4096/ABCDEF1234567890 2026-06-16 [SC]
   ```

   The part after the slash is the key ID: `ABCDEF1234567890`.

3. Export the keys:

   ```bash
   KEYID=ABCDEF1234567890
   gpg --export-secret-keys --armor "$KEYID" > apt-signing-key.asc
   gpg --export "$KEYID" > KEY.gpg
   ```

### Add GitHub Secrets

Go to **Settings → Secrets and variables → Actions → New repository secret** and add:

| Secret | Value |
|---|---|
| `APT_GPG_PRIVATE_KEY` | Copy-paste the **entire** content of `apt-signing-key.asc` (including `-----BEGIN PGP PRIVATE KEY BLOCK-----` and `-----END ...`). |
| `APT_GPG_KEY_ID` | The key ID from step 2, e.g. `ABCDEF1234567890`. |
| `APT_GPG_PASSPHRASE` | The passphrase you entered during key generation. **Skip this secret if the key has no passphrase.** |

### Publish a new version

1. Bump the Debian version in `base/ratatui-todo-list/debian/changelog`. For example, add a new top entry:

   ```text
   ratatui-todo-list (0.2.0-3) unstable; urgency=medium

     * Rebuild against Debian Trixie.

    -- Artem <artem@example.com>  Tue, 16 Jun 2026 15:11:19 +0000
   ```

2. Commit and push to `main`:

   ```bash
   git add base/ratatui-todo-list/debian/changelog
   git commit -m "chore: bump ratatui-todo-list to 0.2.0-3"
   git push origin main
   ```

GitHub Actions will detect the changed package, build the `.deb` and publish it to the shared APT repository on `gh-pages`.

## License

FIXME: add license information
