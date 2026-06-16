# ratatui-todo-list

Terminal todo-list application built with [ratatui](https://github.com/ratatui/ratatui).

## Install from APT repository

### 1. Add the signing key

```bash
sudo wget -qO /usr/share/keyrings/ratatui-todo-list.gpg \
  https://<owner>.github.io/ratatui-todo-list/KEY.gpg
```

### 2. Add the repository

```bash
echo "deb [signed-by=/usr/share/keyrings/ratatui-todo-list.gpg] \
  https://<owner>.github.io/ratatui-todo-list stable main" | \
  sudo tee /etc/apt/sources.list.d/ratatui-todo-list.list
```

### 3. Install

```bash
sudo apt update
sudo apt install ratatui-todo-list
```

Replace `<owner>` with your GitHub username or organization.

## Build from source

```bash
cargo build --release
./target/release/ratatui-todo-list
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

```bash
gpg --full-generate-key
# Select RSA (sign only), 4096 bits, no expiration (or your choice).
# Remember the key ID printed at the end.
```

Export keys:

```bash
KEYID=YOUR_KEY_ID_HERE
gpg --export-secret-keys --armor "$KEYID" > apt-signing-key.asc
gpg --export "$KEYID" > KEY.gpg
```

### Add GitHub Secrets

Go to **Settings → Secrets and variables → Actions** and add:

- `APT_GPG_PRIVATE_KEY`: full content of `apt-signing-key.asc`.
- `APT_GPG_KEY_ID`: the GPG key ID (e.g., `A1B2C3D4E5F67890`).
- `APT_GPG_PASSPHRASE`: (optional) passphrase of the private key. If set, `gpg-preset-passphrase` must be available in the runner.

### Trigger a release

Push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will build the `.deb` and publish the APT repository to `gh-pages`.

## License

FIXME: add license information
