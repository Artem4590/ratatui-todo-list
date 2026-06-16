# ratatui-todo-list

Terminal todo-list application built with [ratatui](https://github.com/ratatui/ratatui).

## Install from APT repository

### 1. Add the signing key

```bash
sudo wget -qO /usr/share/keyrings/ratatui-todo-list.gpg https://Artem4590.github.io/ratatui-todo-list/KEY.gpg
```

### 2. Add the repository

```bash
echo "deb [signed-by=/usr/share/keyrings/ratatui-todo-list.gpg] https://Artem4590.github.io/ratatui-todo-list stable main" | sudo tee /etc/apt/sources.list.d/ratatui-todo-list.list
```

### 3. Install

```bash
sudo apt update
sudo apt install ratatui-todo-list
```

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

Go to **Settings → Secrets and variables → Actions → New repository secret** and add three secrets:

| Secret | Value |
|---|---|
| `APT_GPG_PRIVATE_KEY` | Copy-paste the **entire** content of `apt-signing-key.asc` (including `-----BEGIN PGP PRIVATE KEY BLOCK-----` and `-----END ...`). |
| `APT_GPG_KEY_ID` | The key ID from step 2, e.g. `ABCDEF1234567890`. |
| `APT_GPG_PASSPHRASE` | The passphrase you entered during key generation. **Skip this secret if the key has no passphrase.** |

### Trigger a release

Push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will build the `.deb` and publish the APT repository to `gh-pages`.

## License

FIXME: add license information
