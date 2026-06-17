# Monorepo APT-репозиторий Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Перестроить репозиторий под монорепозиторную модель: перенести проект в `base/ratatui-todo-list/`, оставить в корне только CI/CD и документацию, реализовать общий GitHub Actions workflow для сборки изменённых пакетов и публикации общего APT-репозитория.

**Architecture:** Один общий workflow с тремя job'ами: `detect-packages` (находит изменённые пакеты в `base/`), `build-packages` (matrix-сборка каждого пакета в Docker), `publish-repo` (последовательная публикация в `gh-pages` через reprepro). Публикация выполняется только на push в `main`, PR только собирает.

**Tech Stack:** GitHub Actions, Docker, `reprepro`, GPG, Debian packaging.

---

## File Structure (target)

```text
.
├── .github/
│   └── workflows/
│       └── apt-repo.yml          # общий workflow
├── .gitignore
├── README.md
└── base/
    └── ratatui-todo-list/
        ├── Cargo.toml
        ├── Cargo.lock
        ├── src/
        ├── debian/
        ├── Dockerfile
        └── .dockerignore
```

---

## Task 1: Перенести файлы проекта в `base/ratatui-todo-list/`

**Files:**
- Move: `Cargo.toml`, `Cargo.lock`, `src/`, `debian/`, `Dockerfile`, `.dockerignore` → `base/ratatui-todo-list/`
- Modify: `.gitignore` (пути, если нужно)
- Keep in root: `.github/`, `.gitignore`, `README.md`, `docs/`

- [ ] **Step 1: Создать директорию `base/ratatui-todo-list/` и перенести файлы**

Run:
```bash
mkdir -p base/ratatui-todo-list
git mv Cargo.toml Cargo.lock src debian Dockerfile .dockerignore base/ratatui-todo-list/
```

- [ ] **Step 2: Проверить структуру**

Run:
```bash
find base/ratatui-todo-list -maxdepth 2 -type f | sort
```

Expected:
```text
base/ratatui-todo-list/Cargo.toml
base/ratatui-todo-list/Dockerfile
base/ratatui-todo-list/src/main.rs
base/ratatui-todo-list/debian/changelog
...
```

- [ ] **Step 3: Commit**

```bash
git add .
git commit -m "chore: move ratatui-todo-list package into base/ratatui-todo-list"
```

---

## Task 2: Обновить Dockerfile для работы из поддиректории

**Files:**
- Modify: `base/ratatui-todo-list/Dockerfile`

- [ ] **Step 1: Убедиться, что Dockerfile использует корректный контекст**

Текущий Dockerfile должен работать, потому что его контекст при сборке будет `base/ratatui-todo-list/`. Убедиться, что `COPY . .` копирует именно файлы пакета.

Current `base/ratatui-todo-list/Dockerfile`:
```dockerfile
FROM debian:trixie-slim

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential debhelper fakeroot curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

WORKDIR /build
COPY . .

RUN dpkg-buildpackage -us -uc -b

RUN mkdir -p /out && cp /ratatui-todo-list_*.deb /out/

CMD ["sh", "-c", "cp /out/*.deb /output/"]
```

- [ ] **Step 2: Собрать локально из нового пути**

Run:
```bash
docker build -t ratatui-todo-list-deb base/ratatui-todo-list
```

Expected: `SUCCESS`.

- [ ] **Step 3: Commit (если были изменения)**

```bash
git add base/ratatui-todo-list/Dockerfile
git commit -m "build: adjust Dockerfile for monorepo subdirectory" || echo "No changes"
```

---

## Task 3: Переписать `.github/workflows/apt-repo.yml` под общий monorepo workflow

**Files:**
- Modify: `.github/workflows/apt-repo.yml`

- [ ] **Step 1: Реализовать job `detect-packages`**

```yaml
jobs:
  detect-packages:
    runs-on: ubuntu-latest
    outputs:
      packages: ${{ steps.detect.outputs.packages }}
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Detect changed packages
        id: detect
        run: |
          if [ "${{ github.event_name }}" = "pull_request" ]; then
            base_sha="${{ github.event.pull_request.base.sha }}"
          elif [ "${{ github.event_name }}" = "push" ]; then
            base_sha="${{ github.event.before }}"
          else
            base_sha="HEAD~1"
          fi
          packages=$(git diff --name-only "$base_sha" HEAD \
            | grep '^base/' \
            | cut -d/ -f2 \
            | sort -u \
            | jq -R . \
            | jq -s -c .)
          echo "packages=$packages" >> "$GITHUB_OUTPUT"
          echo "Detected packages: $packages"
```

- [ ] **Step 2: Реализовать job `build-packages` с matrix**

```yaml
  build-packages:
    runs-on: ubuntu-latest
    needs: detect-packages
    if: ${{ needs.detect-packages.outputs.packages != '[]' }}
    strategy:
      matrix:
        package: ${{ fromJson(needs.detect-packages.outputs.packages) }}
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Read package version
        id: version
        run: |
          version=$(head -1 "base/${{ matrix.package }}/debian/changelog" | grep -oP '\(\K[^)]+')
          echo "version=$version" >> "$GITHUB_OUTPUT"
          echo "Package: ${{ matrix.package }}, version: $version"

      - name: Build Debian package
        run: docker build -t "${{ matrix.package }}-deb" "base/${{ matrix.package }}"

      - name: Extract .deb from image
        run: |
          cid=$(docker create "${{ matrix.package }}-deb")
          docker cp "$cid:/out" ./out
          docker rm "$cid"
          mv ./out/*.deb ./
          rm -rf ./out
          ls -lh *.deb

      - name: Upload .deb artifact
        uses: actions/upload-artifact@v4
        with:
          name: deb-${{ matrix.package }}
          path: "*.deb"
```

- [ ] **Step 3: Реализовать job `publish-repo`**

```yaml
  publish-repo:
    runs-on: ubuntu-latest
    needs: build-packages
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    permissions:
      contents: write
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: ./artifacts
          pattern: deb-*

      - name: Install reprepro and gnupg
        run: |
          sudo apt-get update
          sudo apt-get install -y reprepro gnupg

      - name: Configure GPG agent
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

      - name: Cache passphrase for reprepro
        env:
          APT_GPG_PASSPHRASE: ${{ secrets.APT_GPG_PASSPHRASE }}
        run: |
          if [ -n "$APT_GPG_PASSPHRASE" ]; then
            KEYGRIP=$(gpg --list-secret-keys --with-colons | awk -F: '/^grp/ {print $10; exit}')
            PRESET=$(command -v gpg-preset-passphrase || find /usr/lib -name gpg-preset-passphrase 2>/dev/null | head -1)
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

      - name: Verify and publish packages
        run: |
          for pkg_dir in ./artifacts/deb-*; do
            pkg=$(basename "$pkg_dir" | sed 's/^deb-//')
            version=$(head -1 "base/$pkg/debian/changelog" | grep -oP '\(\K[^)]+')
            echo "Publishing $pkg version $version"
            existing=$(reprepro -b repo list stable "$pkg" | grep "${pkg} ${version}" || true)
            if [ -n "$existing" ]; then
              echo "::error::Version ${version} of ${pkg} already exists in APT repository."
              exit 1
            fi
            deb_file=$(find "$pkg_dir" -maxdepth 1 -name "${pkg}_*.deb" -print -quit)
            reprepro -b repo includedeb stable "$deb_file"
          done

      - name: Commit and push gh-pages
        run: |
          cd repo
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add .
          git commit -m "chore(apt): update repository" || echo "No changes"
          git push origin gh-pages
```

- [ ] **Step 4: Проверить YAML**

Run:
```bash
docker run --rm -v "$PWD:/repo" rhysd/actionlint:latest -color /repo/.github/workflows/apt-repo.yml
```

Expected: `no error`.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/apt-repo.yml
git commit -m "ci: rewrite workflow as monorepo-aware APT publisher"
```

---

## Task 4: Обновить корневой `.gitignore` и `README.md`

**Files:**
- Modify: `.gitignore`
- Modify: `README.md`

- [ ] **Step 1: Обновить `.gitignore`**

Добавить общие игнорируемые пути:

```text
/target
*.deb
.worktrees/
```

Если какие-то пути уже есть, оставить без изменений.

- [ ] **Step 2: Обновить `README.md`**

- Указать, что репозиторий — монорепозиторий пакетов.
- Описать структуру `base/<package>/`.
- Обновить инструкции по установке (общий APT-репозиторий).
- Указать, что для релиза нужно обновить `base/<package>/debian/changelog` и запушить в `main`.

- [ ] **Step 3: Commit**

```bash
git add .gitignore README.md
git commit -m "docs: update root docs for monorepo structure"
```

---

## Task 5: Обновить `AGENTS.md`

**Files:**
- Modify: `AGENTS.md`

- [ ] **Step 1: Обновить разделы структуры и CI/CD**

Отразить:
- новую структуру с `base/ratatui-todo-list/`;
- общий workflow;
- отказ от тегов в пользу push в `main`;
- версию из `debian/changelog`.

- [ ] **Step 2: Commit**

```bash
git add AGENTS.md
git commit -m "docs: update AGENTS.md for monorepo layout"
```

---

## Task 6: Локальная проверка

**Files:**
- Read-only: `base/ratatui-todo-list/Dockerfile`, `.github/workflows/apt-repo.yml`

- [ ] **Step 1: Проверить сборку из нового пути**

Run:
```bash
docker build -t ratatui-todo-list-deb base/ratatui-todo-list
```

Expected: `SUCCESS`.

- [ ] **Step 2: Проверить YAML workflow**

Run:
```bash
docker run --rm -v "$PWD:/repo" rhysd/actionlint:latest -color /repo/.github/workflows/apt-repo.yml
```

Expected: `no error`.

- [ ] **Step 3: Проверить git status**

Run:
```bash
git status --short
```

Expected: чистое состояние или только ожидаемые неотслеживаемые файлы.

---

## Task 7: Публикация в репозиторий

**Files:**
- Read-only: вся ветка `feature/apt-repo-ci`

- [ ] **Step 1: Push ветки**

```bash
git push origin feature/apt-repo-ci
```

- [ ] **Step 2: Создать PR в `main` (по желанию)**

- [ ] **Step 3: После мержа — проверить запуск workflow на push main**

Open GitHub Actions tab and confirm:
- `detect-packages` finds `ratatui-todo-list`;
- `build-packages` builds successfully;
- `publish-repo` publishes to `gh-pages`.

---

## Spec Coverage Check

| Spec Requirement | Implementing Task |
|---|---|
| Структура `base/<package>/` | Task 1 |
| Корень содержит только `.github`, `.gitignore`, `README.md` | Task 1, 4 |
| Общий workflow | Task 3 |
| Определение изменённых пакетов | Task 3 |
| Сборка в Docker из поддиректории | Task 2, 3 |
| Публикация только на push main | Task 3 |
| PR build без публикации | Task 3 |
| Версия из `debian/changelog` | Task 3 |
| Обновление документации | Task 4, 5 |

## Placeholder Scan

- `<owner>` в README — допустимый placeholder.
- `<package>` в workflow — переменная matrix, не placeholder.
- В плане нет TODO/TBD.
