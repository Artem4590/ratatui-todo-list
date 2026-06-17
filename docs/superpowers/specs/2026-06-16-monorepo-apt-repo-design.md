# Monorepo APT-репозиторий — Design

## Контекст

Проект `ratatui-todo-list` — экспериментальный репозиторий для отработки упаковки Rust-приложений в `.deb` и публикации APT-репозитория. В дальнейшем подход будет перенесён в общий монорепозиторий, где будут собираться несколько пакетов (до 15 штук) в один общий APT-репозиторий на GitHub Pages.

## Цель

Перестроить текущий репозиторий под монорепозиторную модель:

- Каждый пакет — это директория, содержащая подпапку `debian/`.
- В корне репозитория — только `.github/`, `.gitignore`, `README.md` и вспомогательные файлы.
- Один общий GitHub Actions workflow собирает только изменённые пакеты и публикует их в общий APT-репозиторий на `gh-pages`.
- Версия каждого пакета берётся из его `debian/changelog`.
- Сборка запускается на PR (без публикации), публикация — только при push в `main`.

## Требования

1. Структура репозитория (пример):
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

   Пакетом считается любая директория, внутри которой есть `debian/`.

2. Workflow должен:
   - запускаться на `push` в `main`, на `pull_request` в `main` и вручную (`workflow_dispatch`);
   - находить все директории с `debian/` и проверять, изменились ли файлы внутри каждой из них;
   - собирать каждый изменённый пакет в Docker (`<package-path>/Dockerfile`);
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
│ detect-packages │  ← find все debian/, git diff, ищет изменённые пакеты
└────────┬────────┘
         │ matrix of changed package paths
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
  - для push: `github.event.before`;
  - для PR: `github.event.pull_request.base.sha`;
  - для manual: `HEAD~1`.
- Находит все пакеты среди отслеживаемых git файлов:
  ```bash
  git ls-files | grep -E '/debian/' | sed 's|/debian/.*||' | sort -u
  ```
- Для каждого найденного пакета проверяет, есть ли изменения:
  ```bash
  git diff --name-only <base-sha> HEAD | grep -q "^<package-path>/"
  ```
- Формирует JSON-матрицу из **полных путей** к изменённым пакетам.

### Job `build-packages`

- Matrix по путям пакетов из `detect-packages`.
- Для каждого пакета:
  - вычисляет slug (`tr '/' '-'`);
  - `docker build -t pkg-<slug> <package-path>`;
  - извлекает `.deb` из образа;
  - читает версию из `<package-path>/debian/changelog`;
  - сохраняет путь к пакету в `package-path.txt`;
  - загружает артефакт `deb-<slug>`.

### Job `publish-repo`

- Выполняется только при `github.event_name == 'push'`.
- Зависит от `build-packages`.
- Скачивает все артефакты.
- Устанавливает `reprepro` и `gnupg`.
- Импортирует GPG-ключ.
- Чекаутит `gh-pages`.
- Генерирует/обновляет `conf/distributions`.
- Для каждого артефакта:
  - читает исходный путь пакета из `package-path.txt`;
  - определяет имя пакета через `dpkg-deb -f <deb> Package`;
  - читает версию из `<package-path>/debian/changelog`;
  - проверяет, что версия ещё не в репозитории;
  - `reprepro -b repo includedeb stable <deb>`.
- Коммитит и пушит `gh-pages`.

## Версионирование

- Источник правды: `<package-path>/debian/changelog`.
- CI читает первую строку:
  ```bash
  version=$(head -1 <package-path>/debian/changelog | grep -oP '\(\K[^)]+')
  ```
- Имя пакета для проверки `reprepro list` берётся из самого `.deb`:
  ```bash
  pkg_name=$(dpkg-deb -f <deb-file> Package)
  ```
- Если версия уже есть в APT-репозитории, workflow падает с понятной ошибкой.
- `Cargo.toml` обновляется вручную для консистентности, но CI не использует его как источник версии.

## Безопасность и изоляция

- Каждый пакет собирается в своём Docker-контексте (`<package-path>/`).
- Публикация в `gh-pages` последовательная, чтобы избежать конфликтов пушей.
- GPG-ключ хранится в GitHub Secrets.

## Критерии успеха

1. Workflow успешно находит пакет по наличию `debian/` и собирает его.
2. Push в `main` публикует `.deb` в `gh-pages`.
3. PR запускает только сборку, без публикации.
4. Добавление нового пакета в любую директорию с `debian/` не требует изменений в workflow, кроме наличия `Dockerfile` и `debian/changelog`.
