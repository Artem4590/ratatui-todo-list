# AGENTS.md — ratatui-todo-list

Файл для AI-агентов, работающих с проектом. Проект — небольшое TUI-приложение (terminal user interface) для ведения списка задач на Rust с использованием библиотеки [ratatui](https://github.com/ratatui/ratatui).

## Обзор проекта

- **Название:** `ratatui-todo-list`
- **Версия:** `0.2.0` (см. `Cargo.toml` и `debian/changelog`)
- **Язык разработки:** Rust, edition `2024`
- **Тип приложения:** консольное (TUI), один исполняемый бинарник
- **Основное назначение:** интерактивный todo-list в терминале
- **Лицензия:** в `debian/copyright` указана MIT, но полный текст лицензии не добавлен (`FIXME: add full license text`), в `README.md` тоже `FIXME: add license information`

Проект собирается как обычный Rust-бинарник, а также упаковывается в `.deb` для Debian/Ubuntu. Репозиторий оформлен как **монорепозиторий**: каждый пакет живёт в `base/<package>/`, а в корне находится общий CI/CD, который собирает изменённые пакеты и публикует их в общий APT-репозиторий на GitHub Pages.

## Технологический стек

- **Язык и инструменты:** Rust, Cargo
- **TUI-фреймворк:** `ratatui = "0.30.0"`
- **Ввод с терминала:** `crossterm = "0.29.0"` (используется как зависимость `ratatui`, события клавиатуры обрабатываются через `ratatui::crossterm::event`)
- **Обработка ошибок:** `color-eyre = "0.6.5"` — добавлена в зависимости, но в `src/main.rs` пока не используется
- **Сериализация:** `serde` + `serde_json` — структуры для сохранения состояния объявлены, но механизм загрузки/сохранения не реализован
- **Пути к данным пользователя:** `directories = "6.0.0"` — добавлена, но пока не используется
- **Сборка Debian-пакета:** `dpkg-buildpackage`, `debhelper`, Docker
- **CI/CD:** GitHub Actions
- **APT-репозиторий:** `reprepro`, GPG-подпись, публикация через ветку `gh-pages` и GitHub Pages

## Структура проекта

```text
.
├── .github/workflows/
│   └── apt-repo.yml        # CI/CD: общий workflow для всех пакетов
├── .gitignore
├── README.md               # Пользовательская документация (английский)
├── AGENTS.md               # Этот файл
├── docs/superpowers/       # Внутренние спецификации и планы (русский язык)
│   ├── specs/2026-06-16-github-actions-apt-repo-design.md
│   ├── specs/2026-06-16-monorepo-apt-repo-design.md
│   ├── plans/2026-06-16-github-actions-apt-repo.md
│   └── plans/2026-06-16-monorepo-apt-repo.md
└── base/
    └── ratatui-todo-list/  # Исходники и упаковка одного пакета
        ├── Cargo.toml
        ├── Cargo.lock
        ├── src/
        │   └── main.rs
        ├── debian/
        │   ├── changelog
        │   ├── compat
        │   ├── control
        │   ├── copyright
        │   ├── rules
        │   └── source/format
        ├── Dockerfile
        └── .dockerignore
```

### Главный модуль — `base/ratatui-todo-list/src/main.rs`

Весь код находится в одном файле. Основные сущности:

| Сущность | Назначение |
|----------|------------|
| `App` | Главное состояние приложения: флаг выхода, список задач, режим ввода, состояние выделения в `List` |
| `TodoList` | Список задач (`Vec<TodoItem>`) |
| `TodoItem` | Одна задача: текст + статус |
| `Status` | `Todo` / `Completed` |
| `Mode` | `Normal` / `Adding` / `Editing` — текущий режим интерфейса |
| `PersistedState` | **Заготовка** для сохранения/загрузки состояния (сейчас не используется) |

Главный цикл приложения:

1. `main()` создаёт `App::default()` и запускает `ratatui::run(|terminal| app.run(terminal))`.
2. `App::run()` в цикле рисует интерфейс (`draw`) и обрабатывает события клавиатуры (`handle_events`).
3. `draw()` только читает `self` и отрисовывает виджеты; важно не менять состояние внутри отрисовки.
4. `handle_events()` / `handle_key_event()` меняют состояние в зависимости от нажатий.

### Управление в приложении

| Режим | Клавиша | Действие |
|-------|---------|----------|
| Normal | `q` | Выход |
| Normal | `e` | Переход в режим редактирования выделенной задачи |
| Normal | `Ctrl + a` | Вставить новую пустую задачу под текущей и перейти в режим `Adding` |
| Normal | `Ctrl + d` | Удалить выделенную задачу |
| Normal | `Space` | Переключить статус задачи (`Todo` ↔ `Completed`) |
| Normal | `↑` / `↓` | Навигация по списку |
| Adding / Editing | печатаемые символы | Добавить символ в конец текста задачи |
| Adding / Editing | `Backspace` | Удалить последний символ |
| Adding / Editing | `Esc` | Вернуться в `Normal` |

## Команды сборки и запуска

### Локальная разработка

```bash
cd base/ratatui-todo-list

# Сборка отладочной версии
cargo build

# Сборка релизной версии
cargo build --release

# Запуск
cargo run

# Или после релизной сборки:
./target/release/ratatui-todo-list
```

### Сборка Debian-пакета

```bash
cd base/ratatui-todo-list

# Локально (требуется Debian/Ubuntu с debhelper)
dpkg-buildpackage -us -uc -b

# Или через Docker
docker build -t ratatui-todo-list-deb .
```

Dockerfile использует `debian:trixie-slim`, устанавливает Rust через `rustup`, копирует исходники и вызывает `dpkg-buildpackage -us -uc -b`. Готовый `.deb` копируется в `/out`.

### Проверка и форматирование

```bash
# Проверка форматирования
cargo fmt -- --check

# Автоформатирование
cargo fmt

# Проверка предупреждений компилятора
cargo clippy

# Запуск тестов (сейчас тестов нет, но команда корректна)
cargo test
```

## Тестирование

- **Юнит-тесты в проекте отсутствуют.** `cargo test` завершается успешно с `running 0 tests`.
- Основная логика (переключение статусов, добавление/удаление задач, навигация) пока не покрыта тестами.
- Если будете добавлять тесты, рекомендуется:
  - Вынести чистую логику состояния (`TodoList`, `TodoItem`, переключение статуса, добавление/удаление) в отдельные функции/методы и тестировать их напрямую.
  - UI- и ввод с клавиатуры тестировать отдельно или вручную, потому что они зависят от терминала.

## Стиль кода и соглашения

- **Язык комментариев и документации:** русский. Все doc-комментарии и пояснения в `src/main.rs` написаны по-русски.
- **Разделение ответственности:**
  - Состояние хранится в структурах (`App`, `TodoList`, `TodoItem`).
  - Отрисовка (`draw`) только читает состояние и строит виджеты.
  - Обработка событий меняет состояние.
- **Именование:** `PascalCase` для типов/структур/перечислений, `snake_case` для функций и переменных, `SCREAMING_SNAKE_CASE` для констант (`SELECTED_STYLE`).
- **Предупреждения:** в текущей версии `cargo build` / `cargo test` выдаёт предупреждения о неиспользуемых импортах и структурах (`directories`, `ProjectDirs`, `Path`, `PathBuf`, `fs`, `vec`, `PersistedState`). Это обрывки незавершённой функциональности сохранения состояния.

## Развёртывание и релизы

### Автоматический CI/CD

Файл `.github/workflows/apt-repo.yml` — общий workflow `APT Repository`:

- **Триггеры:**
  - Пуш в `main`
  - Pull request в `main`
  - Ручной запуск (`workflow_dispatch`)
- **Job `detect-packages`:**
  - Сравнивает текущий коммит с базовым (`github.event.before` для push, base SHA для PR).
  - Находит изменённые папки в `base/`.
  - Формирует JSON-матрицу пакетов для сборки.
- **Job `build-packages`:**
  - Запускается на `ubuntu-latest`.
  - Matrix по изменённым пакетам.
  - Собирает Docker-образ `base/<package>/Dockerfile`.
  - Извлекает `.deb` и загружает артефакт `deb-<package>`.
- **Job `publish-repo` (только push в `main`):**
  - Скачивает все артефакты.
  - Устанавливает `reprepro` и `gnupg`.
  - Импортирует GPG-ключ из секрета `APT_GPG_PRIVATE_KEY`.
  - Опционально кэширует пароль из секрета `APT_GPG_PASSPHRASE`.
  - Чекаутит ветку `gh-pages` в папку `repo`.
  - Создаёт `repo/conf/distributions` для `reprepro`.
  - Экспортирует публичный ключ в `repo/KEY.gpg`.
  - Для каждого пакета читает версию из `base/<package>/debian/changelog`.
  - Проверяет, что версия пакета ещё не опубликована.
  - Последовательно добавляет пакеты в APT-репозиторий и пушит изменения в `gh-pages`.

### Необходимые секреты репозитория

| Секрет | Назначение |
|--------|------------|
| `APT_GPG_PRIVATE_KEY` | Приватный GPG-ключ для подписи репозитория |
| `APT_GPG_PASSPHRASE` | Пароль ключа (опционально) |
| `GITHUB_TOKEN` | Стандартный токен GitHub Actions для пуша в `gh-pages` |

### Публикация APT-репозитория

1. В настройках репозитория включить GitHub Pages для ветки `gh-pages`, корень `/`.
2. Обновить версию пакета в `base/<package>/debian/changelog`.
3. Закоммитить и запушить в `main`:
   ```bash
   git add base/ratatui-todo-list/debian/changelog
   git commit -m "chore: bump ratatui-todo-list to 0.2.0-3"
   git push origin main
   ```
4. GitHub Actions определит изменённые пакеты, соберёт `.deb` и опубликует их в общий репозиторий.
5. Пользователи добавляют ключ и репозиторий по инструкции из `README.md` и устанавливают пакет:
   ```bash
   sudo apt update
   sudo apt install ratatui-todo-list
   ```

### Debian-метаданные

Файлы расположены в `base/ratatui-todo-list/debian/`:

- `debian/control` — описание пакета, зависимости сборки/установки.
- `debian/changelog` — история версий (`0.1.0-1`, `0.2.0-1`, `0.2.0-2`).
- `debian/rules` — переопределения для `dh`:
  - `override_dh_auto_build`: `cargo build --release`
  - `override_dh_auto_install`: копирует бинарник в `debian/ratatui-todo-list/usr/bin/`
  - `override_dh_auto_clean`: `cargo clean`
- `debian/compat` — уровень `debhelper`: `13`.
- `debian/source/format` — формат пакета: `3.0 (native)`.

## Безопасность

- **GPG-ключ:** приватная часть хранится только в GitHub Secrets. Публичная часть автоматически публикуется как `KEY.gpg` в `gh-pages`.
- **Пароль ключа:** опционален; если задан `APT_GPG_PASSPHRASE`, workflow использует `gpg-preset-passphrase` и loopback pinentry.
- **Dockerfile:** устанавливает Rust через официальный скрипт `rustup`; образ основан на `debian:trixie-slim`.
- **Ввод пользователя:** текст задач вводится без валидации и фильтрации управляющих символов (кроме `Esc`). Это локальное TUI-приложение, но если в будущем появится сохранение в файл, стоит проверять данные.
- **Права:** workflow job `publish-repo` запрашивает `permissions: contents: write` для пуша в `gh-pages`.

## Известные особенности и TODO

- **Сохранение состояния не реализовано.** Объявлены `PersistedState`, `serde`-производные, подключены `directories` и `serde_json`, но код загрузки/сохранения отсутствует. При каждом запуске приложение начинается с четырёх встроенных задач.
- **Нет тестов.** Любые изменения логики состояния стоит сопровождать юнит-тестами.
- **Лицензия не заполнена.** В `debian/copyright` и `README.md` есть `FIXME` по лицензии.
- **README содержит placeholder `<owner>`**, который мейнтейнер должен заменить на имя пользователя/организации GitHub перед публикацией.

## Полезные команды для агента

```bash
# Быстрая проверка проекта
cd base/ratatui-todo-list && cargo check

# Проверка форматирования и стиля
cd base/ratatui-todo-list && cargo fmt -- --check && cargo clippy -- -D warnings

# Сборка и запуск
cd base/ratatui-todo-list && cargo run

# Сборка релизного бинарника
cd base/ratatui-todo-list && cargo build --release

# Сборка .deb в Docker
docker build -t ratatui-todo-list-deb base/ratatui-todo-list

# Проверка workflow YAML (если есть docker и actionlint)
docker run --rm -v "$PWD:/repo" rhysd/actionlint:latest -color /repo/.github/workflows/apt-repo.yml
```
