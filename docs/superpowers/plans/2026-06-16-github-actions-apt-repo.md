# GitHub Actions APT-репозиторий Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Настроить GitHub Actions workflow, который при пуше тега `v*` собирает `.deb` в Docker и публикует подписанный APT-репозиторий на GitHub Pages (ветка `gh-pages`) через `reprepro`.

**Architecture:** Двухjob-овый workflow: `build-deb` собирает пакет в Docker и выгружает артефакт; `update-apt-repo` устанавливает `reprepro`/`gnupg`, импортирует GPG-ключ, обновляет ветку `gh-pages` и публикует `KEY.gpg` + индексы. Пользователь добавляет репозиторий через `sources.list` и ставит пакет `apt install ratatui-todo-list`.

**Tech Stack:** GitHub Actions, Docker (`rust:1.96-bookworm`), `dpkg-buildpackage`, `reprepro`, GPG, GitHub Pages.

---

## File Structure

| File | Responsibility |
|---|---|
| `.github/workflows/apt-repo.yml` | CI/CD: сборка `.deb` и публикация APT-репозитория |
| `README.md` | Инструкции по настройке GPG-секретов, включению Pages, установке через `apt` |
| `Dockerfile` | Уже существует; используется workflow для сборки `.deb` |
| `debian/*` | Уже существуют; metadata для `dpkg-buildpackage` |
| `docs/superpowers/specs/2026-06-16-github-actions-apt-repo-design.md` | Спецификация, по которой строится план |

---

## Task 1: Создать GitHub Actions workflow — сборка `.deb`

**Files:**
- Create: `.github/workflows/apt-repo.yml`

- [ ] **Step 1: Создать файл workflow с job `build-deb`**

```yaml
name: Build and publish APT repository

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build-deb:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build Debian package in Docker
        run: docker build -t ratatui-todo-list-deb .

      - name: Extract .deb from image
        run: |
          cid=$(docker create ratatui-todo-list-deb)
          docker cp "$cid:/out" ./out
          docker rm "$cid"
          mv ./out/ratatui-todo-list_*.deb ./
          rmdir ./out || true
          ls -lh ratatui-todo-list_*.deb

      - name: Upload .deb artifact
        uses: actions/upload-artifact@v4
        with:
          name: deb-package
          path: ratatui-todo-list_*.deb
```

- [ ] **Step 2: Проверить YAML-синтаксис**

Run:
```bash
docker run --rm -v "$PWD:/repo" rhysd/actionlint:latest -color /repo/.github/workflows/apt-repo.yml
```

Expected: `no error`.

Если `actionlint` недоступен, выполнить как минимум:
```bash
cat .github/workflows/apt-repo.yml
```
и визуально проверить отступы.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/apt-repo.yml
git commit -m "ci: add build-deb job for apt repository"
```

---

## Task 2: Добавить job `update-apt-repo` и GPG-подпись

**Files:**
- Modify: `.github/workflows/apt-repo.yml`

- [ ] **Step 1: Добавить job `update-apt-repo` в конец workflow**

```yaml
  update-apt-repo:
    runs-on: ubuntu-latest
    needs: build-deb
    permissions:
      contents: write
    steps:
      - name: Download .deb artifact
        uses: actions/download-artifact@v4
        with:
          name: deb-package
          path: ./deb

      - name: Install reprepro and gnupg
        run: |
          sudo apt-get update
          sudo apt-get install -y reprepro gnupg

      - name: Configure GPG agent for loopback pinentry
        run: |
          mkdir -p ~/.gnupg
          chmod 700 ~/.gnupg
          echo "allow-loopback-pinentry" > ~/.gnupg/gpg-agent.conf
          gpg-connect-agent reloadagent /bye

      - name: Import GPG signing key
        env:
          APT_GPG_PRIVATE_KEY: ${{ secrets.APT_GPG_PRIVATE_KEY }}
          APT_GPG_PASSPHRASE: ${{ secrets.APT_GPG_PASSPHRASE }}
        run: |
          set -e
          if [ -z "$APT_GPG_PRIVATE_KEY" ]; then
            echo "Error: APT_GPG_PRIVATE_KEY secret is not set" >&2
            exit 1
          fi
          if [ -n "$APT_GPG_PASSPHRASE" ]; then
            echo "$APT_GPG_PRIVATE_KEY" | gpg --batch --yes --pinentry-mode loopback --passphrase "$APT_GPG_PASSPHRASE" --import
          else
            echo "$APT_GPG_PRIVATE_KEY" | gpg --batch --yes --import
          fi
          KEY_ID=$(gpg --list-secret-keys --with-colons | grep '^sec' | head -1 | cut -d: -f5)
          echo "KEY_ID=$KEY_ID" >> "$GITHUB_ENV"
          echo "GPG key imported: $KEY_ID"

      - name: Cache passphrase for reprepro (if protected key)
        env:
          APT_GPG_PASSPHRASE: ${{ secrets.APT_GPG_PASSPHRASE }}
        run: |
          if [ -n "$APT_GPG_PASSPHRASE" ]; then
            KEYGRIP=$(gpg --list-secret-keys --with-colons | awk -F: '/^grp/ {print $10; exit}')
            echo "keygrip=$KEYGRIP"
            PRESET=$(command -v gpg-preset-passphrase || find /usr/lib -name gpg-preset-passphrase 2>/dev/null | head -1)
            if [ -z "$PRESET" ]; then
              echo "gpg-preset-passphrase not found; create an unprotected signing key or install gnupg-agent" >&2
              exit 1
            fi
            "$PRESET" --preset --passphrase "$APT_GPG_PASSPHRASE" "$KEYGRIP"
          fi

      - name: Checkout gh-pages branch
        uses: actions/checkout@v4
        with:
          ref: gh-pages
          path: repo
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Ensure reprepro configuration
        run: |
          mkdir -p repo/conf
          cat > repo/conf/distributions <<EOF
          Origin: ratatui-todo-list
          Label: ratatui-todo-list
          Suite: stable
          Codename: stable
          Architectures: amd64
          Components: main
          Description: APT repository for ratatui-todo-list
          SignWith: ${{ env.KEY_ID }}
          EOF
          gpg --export "${{ env.KEY_ID }}" > repo/KEY.gpg

      - name: Add .deb to APT repository
        run: |
          reprepro -b repo includedeb stable ./deb/ratatui-todo-list_*.deb

      - name: Commit and push gh-pages
        run: |
          cd repo
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add .
          git commit -m "chore(apt): update repository for ${{ github.ref_name }}" || echo "No changes to commit"
          git push origin gh-pages
```

- [ ] **Step 2: Проверить YAML-синтаксис**

Run:
```bash
docker run --rm -v "$PWD:/repo" rhysd/actionlint:latest -color /repo/.github/workflows/apt-repo.yml
```

Expected: `no error`.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/apt-repo.yml
git commit -m "ci: add update-apt-repo job with reprepro and gpg signing"
```

---

## Task 3: Добавить инструкции в README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Добавить раздел "Install from APT repository"**

```markdown
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
```

- [ ] **Step 2: Добавить раздел "Maintainer setup"**

```markdown
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
- `APT_GPG_PASSPHRASE`: (optional) passphrase of the private key.

### Trigger a release

Push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions will build the `.deb` and publish the APT repository.
```

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add APT repository setup and install instructions"
```

---

## Task 4: Локальная проверка workflow (pre-flight)

**Files:**
- Read-only: `.github/workflows/apt-repo.yml`, `README.md`

- [ ] **Step 1: Проверить, что Dockerfile всё ещё собирает `.deb`**

Run:
```bash
docker build -t ratatui-todo-list-deb .
```

Expected: `SUCCESS`.

- [ ] **Step 2: Проверить YAML workflow**

Run:
```bash
docker run --rm -v "$PWD:/repo" rhysd/actionlint:latest -color /repo/.github/workflows/apt-repo.yml
```

Expected: `no error`.

- [ ] **Step 3: Проверить, что README не содержит битых placeholder'ов**

Run:
```bash
grep -n "<owner>" README.md
```

Expected: placeholders present and documented (user will replace them).

---

## Task 5: Запуск в репозитории GitHub

**Files:**
- Read-only: `.github/workflows/apt-repo.yml`

- [ ] **Step 1: Push workflow и README в `main`**

```bash
git push origin main
```

- [ ] **Step 2: Настроить GitHub Secrets и Pages**

Follow the **Maintainer setup** section in README:
- create `gh-pages` branch,
- enable Pages from `gh-pages`,
- add `APT_GPG_PRIVATE_KEY`, `APT_GPG_KEY_ID`, and optional `APT_GPG_PASSPHRASE`.

- [ ] **Step 3: Создать тестовый релиз**

```bash
git tag v0.1.0-test
git push origin v0.1.0-test
```

- [ ] **Step 4: Проверить результат в GitHub Actions**

Open `Actions` tab and confirm:
- `build-deb` completes successfully,
- `update-apt-repo` completes successfully,
- `gh-pages` branch contains `KEY.gpg`, `conf/distributions`, `dists/`, and `pool/`.

- [ ] **Step 5: Проверить установку на чистой Debian/Ubuntu**

Run on a fresh Debian/Ubuntu VM/container:
```bash
sudo wget -qO /usr/share/keyrings/ratatui-todo-list.gpg \
  https://<owner>.github.io/ratatui-todo-list/KEY.gpg

echo "deb [signed-by=/usr/share/keyrings/ratatui-todo-list.gpg] \
  https://<owner>.github.io/ratatui-todo-list stable main" | \
  sudo tee /etc/apt/sources.list.d/ratatui-todo-list.list

sudo apt update
sudo apt install ratatui-todo-list
```

Expected: package installs, binary is at `/usr/bin/ratatui-todo-list`.

---

## Spec Coverage Check

| Spec Requirement | Implementing Task |
|---|---|
| Сборка `.deb` при пуше тега `v*` | Task 1 |
| Сборка в Docker (`rust:1.96-bookworm`) | Task 1 (использует существующий Dockerfile) |
| APT-репозиторий через `reprepro` | Task 2 |
| Подпись GPG-ключом | Task 2 |
| Публикация на GitHub Pages (`gh-pages`) | Task 2 |
| Пользовательская инструкция по `apt install` | Task 3 |
| Поддержка `amd64` и компонента `main` | Task 2 (`conf/distributions`) |

## Placeholder Scan

- `<owner>` — допустимый placeholder, заменяется пользователем в README.
- `YOUR_KEY_ID_HERE` — допустимый placeholder, заменяется мейнтейнером.
- В workflow нет TODO/TBD.

## Type/Name Consistency

- Secret names: `APT_GPG_PRIVATE_KEY`, `APT_GPG_KEY_ID`, `APT_GPG_PASSPHRASE` — единообразно.
- Workflow job names: `build-deb`, `update-apt-repo`.
- Artifact name: `deb-package`.
- Branch: `gh-pages`.
