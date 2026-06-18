# GitHub Actions + APT-репозиторий на GitHub Pages

## Цель

Настроить автоматическую сборку `.deb`-пакета в GitHub Actions и публикацию APT-репозитория на GitHub Pages, чтобы пользователи могли добавить репозиторий и установить `ratatui-todo-list` через `apt`.

## Контекст

- Проект: Rust TUI-приложение `ratatui-todo-list`.
- `.deb` уже собирается локально в Docker через `dpkg-buildpackage`.
- Необходимо: CI/CD, signed APT repo, простая установка для конечных пользователей.

## Требования

1. Сборка `.deb` запускается при пуше тега `v*` и вручную (`workflow_dispatch`).
2. Сборка происходит в Docker-контейнере на базе Debian (`rust:1.96-bookworm`).
3. APT-репозиторий ведется через `reprepro` и подписывается GPG-ключом.
4. Репозиторий публикуется на GitHub Pages из ветки `gh-pages`.
5. Пользователь получает инструкцию по добавлению репозитория и ключа.
6. Поддерживается архитектура `amd64` и компонент `main`.

## Архитектура

```
┌─────────────────┐     push tag v*      ┌──────────────────────┐
│   GitHub Repo   │ ───────────────────> │  GitHub Actions      │
└─────────────────┘                      │  .github/workflows/  │
                                         │  apt-repo.yml        │
                                         └──────────┬───────────┘
                                                    │
                       ┌────────────────────────────┼────────────────────────────┐
                       │                            │                            │
                       ▼                            ▼                            ▼
              ┌─────────────────┐         ┌─────────────────┐          ┌─────────────────┐
              │  Docker build   │         │  GPG import     │          │  reprepro       │
              │  .deb artifact  │         │  from secrets   │          │  update repo    │
              └─────────────────┘         └─────────────────┘          └─────────────────┘
                                                                                │
                                                                                ▼
                                                                       ┌─────────────────┐
                                                                       │  gh-pages branch │
                                                                       │  pool/           │
                                                                       │  dists/          │
                                                                       │  KEY.gpg         │
                                                                       └─────────────────┘
                                                                                │
                                                                                ▼
                                                                       ┌─────────────────┐
                                                                       │  GitHub Pages   │
                                                                       │  apt repo URL   │
                                                                       └─────────────────┘
```

## Компоненты

### 1. GitHub Actions workflow (`apt-repo.yml`)

Два job'а:

#### `build-deb`

- `ubuntu-latest` runner.
- Checkout исходников.
- `docker build -t ratatui-todo-list-deb .`
- Извлечение `.deb` из контейнера.
- `actions/upload-artifact` с `.deb`.

#### `update-apt-repo`

- Depends on `build-deb`.
- `actions/download-artifact`.
- Checkout ветки `gh-pages` в подкаталог `repo` (или создание orphan-ветки, если её нет).
- Установка `reprepro` и `gnupg`.
- Импорт GPG private key из secret `APT_GPG_PRIVATE_KEY`.
- Распаковка публичного ключа в `repo/KEY.gpg`.
- Генерация `repo/conf/distributions` с `SignWith: ${{ secrets.APT_GPG_KEY_ID }}`.
- `reprepro -b repo includedeb stable <deb>`.
- Commit & push `gh-pages`.

### 2. GPG-ключ

- Пользователь генерирует ключ локально:
  ```bash
  gpg --full-generate-key
  gpg --export-secret-keys --armor KEYID > apt-signing-key.asc
  gpg --export KEYID > KEY.gpg
  ```
- `apt-signing-key.asc` добавляется в GitHub Secret `APT_GPG_PRIVATE_KEY`.
- `APT_GPG_KEY_ID` добавляется в GitHub Secrets (можно получить через `gpg --list-secret-keys`).
- Ключ используется без passphrase.
- `KEY.gpg` публикуется в корне `gh-pages` автоматически workflow.

### 3. reprepro конфигурация

Файл `repo/conf/distributions`:

```text
Origin: ratatui-todo-list
Label: ratatui-todo-list
Suite: stable
Codename: stable
Architectures: amd64
Components: main
Description: APT repository for ratatui-todo-list
SignWith: <GPG KEY ID>
```

### 4. GitHub Pages

- В настройках репозитория включается Pages для ветки `gh-pages`, папка `/ (root)`.
- После пуша в `gh-pages` репозиторий доступен по `https://<owner>.github.io/ratatui-todo-list/`.

## Инструкция для пользователя

```bash
sudo wget -qO /usr/share/keyrings/ratatui-todo-list.gpg \
  https://<owner>.github.io/ratatui-todo-list/KEY.gpg

echo "deb [signed-by=/usr/share/keyrings/ratatui-todo-list.gpg] \
  https://<owner>.github.io/ratatui-todo-list stable main" | \
  sudo tee /etc/apt/sources.list.d/ratatui-todo-list.list

sudo apt update
sudo apt install ratatui-todo-list
```

## Файлы, которые будут созданы/изменены

- `.github/workflows/apt-repo.yml`
- `README.md` (добавить разделы про релиз и установку через apt)
- `docs/superpowers/specs/2026-06-16-github-actions-apt-repo-design.md` (этот документ)
- Ветка `gh-pages` создается автоматически при первом запуске workflow.

## Ограничения

- GitHub Pages имеет лимит ~1 GB на репозиторий; для небольшого `.deb` (~240 KB) это несущественно.
- GPG-ключ используется без passphrase.
- Workflow требует предварительной настройки secrets вручную.

## Критерии успеха

1. При пуше тега `v*` workflow завершается успешно.
2. В ветке `gh-pages` появляются `dists/`, `pool/` и `KEY.gpg`.
3. `apt update` и `apt install ratatui-todo-list` работают на чистой Debian/Ubuntu-системе.
