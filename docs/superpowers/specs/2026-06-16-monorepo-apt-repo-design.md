# Monorepo APT-репозиторий — Design

## Контекст

Проект `ratatui-todo-list` — экспериментальный репозиторий для отработки упаковки Rust-приложений в `.deb` и публикации APT-репозитория. В дальнейшем подход будет перенесён в общий монорепозиторий, где будут собираться несколько пакетов (до 15 штук) в один общий APT-репозиторий на GitHub Pages.

## Цель

Перестроить текущий репозиторий под монорепозиторную модель:

- Весь код и файлы сборки пакета находятся в `base/<package>/`.
- В корне репозитория — только `.github/`, `.gitignore` и `README.md`.
- Один общий GitHub Actions workflow собирает только изменённые пакеты и публикует их в общий APT-репозиторий на `gh-pages`.
- Версия каждого пакета берётся из его `debian/changelog`.
- Сборка запускается на PR (без публикации), публикация — только при push в `main`.

## Требования

1. Структура репозитория:
   ```text
   .
   ├── .github/
   │   └── workflows/
   │       └── apt-repo.yml
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

2. Workflow должен:
   - запускаться на `push` в `main`, на `pull_request` в `main` и вручную (`workflow_dispatch`);
   - определять, какие пакеты в `base/` изменились;
   - собирать каждый изменённый пакет в Docker (`base/<package>/Dockerfile`);
   - на push в `main` публиковать все собранные `.deb` в общий APT-репозиторий на `gh-pages`;
   - на PR только собирать и проверять, не публиковать.

3. Версия пакета:
   - Берётся из `base/<package>/debian/changelog`.
   - Автоматический bump версии не делается; мейнтейнер обновляет changelog перед мержем.

4. APT-репозиторий:
   - Общий для всех пакетов.
   - Подписывается GPG-ключом.
   - Публикуется через GitHub Pages из ветки `gh-pages`.

## Архитектура

```text
push / PR / manual
        │
        ▼
┌─────────────────┐
│ detect-packages │  ← git diff, ищет изменённые папки в base/
└────────┬────────┘
         │ matrix of changed packages
         ▼
┌──────────────────┐
│ build-packages   │  ← docker build per package, upload artifact
└────────┬─────────┘
         │ on push main only
         ▼
┌──────────────────┐
│ publish-repo     │  ← checkout gh-pages, reprepro includedeb per package, push
└──────────────────┘
         │
         ▼
   gh-pages (GitHub Pages)
```

## Workflow

### Job `detect-packages`

- Определяет base SHA:
  - для push: `HEAD~1` или `github.event.before`;
  - для PR: `github.event.pull_request.base.sha`;
  - для manual: можно задать вручную или использовать `HEAD~1`.
- Выполняет:
  ```bash
  git diff --name-only <base-sha> HEAD \
    | grep '^base/' \
    | cut -d/ -f2 \
    | sort -u
  ```
- Формирует JSON-матрицу для следующего job'а.

### Job `build-packages`

- Matrix по пакетам из `detect-packages`.
- Для каждого пакета:
  - checkout (достаточно shallow);
  - `docker build -t <package>-deb base/<package>`;
  - извлекает `.deb` из образа;
  - читает версию из `base/<package>/debian/changelog`;
  - загружает артефакт `deb-<package>`.

### Job `publish-repo`

- Выполняется только при `github.event_name == 'push'`.
- Зависит от `build-packages`.
- Скачивает все артефакты.
- Устанавливает `reprepro` и `gnupg`.
- Импортирует GPG-ключ.
- Чекаутит `gh-pages`.
- Генерирует/обновляет `conf/distributions`.
- Для каждого пакета:
  - проверяет, что версия из changelog ещё не в репозитории;
  - `reprepro -b repo includedeb stable ./artifacts/<package>_*.deb`.
- Коммитит и пушит `gh-pages`.

## Версионирование

- Источник правды: `base/<package>/debian/changelog`.
- CI читает первую строку:
  ```bash
  version=$(head -1 base/<package>/debian/changelog | grep -oP '\(\K[^)]+')
  ```
- Если версия уже есть в APT-репозитории, workflow падает с понятной ошибкой.
- `Cargo.toml` обновляется вручную для консистентности, но CI не использует его как источник версии.

## Безопасность и изоляция

- Каждый пакет собирается в своём Docker-контексте (`base/<package>/`).
- Публикация в `gh-pages` последовательная, чтобы избежать конфликтов пушей.
- GPG-ключ хранится в GitHub Secrets.

## Критерии успеха

1. После переноса `ratatui-todo-list` в `base/ratatui-todo-list/` workflow успешно собирает пакет.
2. Push в `main` публикует `.deb` в `gh-pages`.
3. PR запускает только сборку, без публикации.
4. Добавление нового пакета в `base/<new-package>/` не требует изменений в workflow, кроме наличия `Dockerfile` и `debian/changelog`.
