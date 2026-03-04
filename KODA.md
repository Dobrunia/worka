# KODA.md — Контекст проекта Worka

## Обзор проекта

**Worka** — десктопное приложение для автоматического трекинга времени и активности на Windows 10. Работает в системном трее, пассивно собирает метаданные о пользователе без ручного ввода.

### Назначение
- Автоматический учёт времени работы за компьютером
- Отслеживание используемых приложений
- Подсчёт активного времени и периодов простоя
- Сбор статистики ввода (количество нажатий клавиш и кликов мыши)
- Визуализация данных через дашборд

### Ключевые принципы
- **Приватность** — все данные хранятся локально в SQLite
- **Минимальное вторжение** — не сохраняется текст, не делаются скриншоты
- **Пассивный сбор** — никаких ручных таймеров

---

## Технологический стек

| Слой | Технология |
|------|------------|
| Desktop shell | Tauri v2 |
| Frontend | Vue 3 + TypeScript + Vite |
| UI библиотека | dobruniaui-vue |
| Графики | Chart.js + vue-chartjs |
| Backend | Rust |
| База данных | SQLite (rusqlite) |
| Тестирование (FE) | Vitest + @testing-library/vue |
| Тестирование (BE) | Cargo test |

---

## Структура проекта

```
worka/
├── src/                    # Vue 3 frontend
│   ├── App.vue            # Корневой компонент с навигацией
│   ├── main.ts            # Точка входа
│   ├── components/
│   │   ├── layout/        # AppHeader, AppNavigation, AppFooter
│   │   └── ui/            # KpiCard, TopAppsList
│   ├── composables/       # Vue composables (useTodayData)
│   ├── utils/             # Утилиты (time.ts)
│   └── views/             # Экраны: TodayView, WeekView, TimelineView, SettingsView
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── lib.rs         # Основная логика, Tauri команды
│   │   └── main.rs        # Точка входа
│   ├── Cargo.toml         # Зависимости Rust
│   └── tauri.conf.json    # Конфигурация Tauri
├── public/                # Статические ассеты
├── package.json           # Зависимости npm, скрипты
├── vite.config.ts         # Конфигурация Vite
└── tsconfig.json          # Конфигурация TypeScript
```

---

## Сборка и запуск

### Разработка

```bash
npm install                # Установка зависимостей
npm run dev                # Запуск Tauri в режиме разработки (Vite + Rust)
npm run dev:vite           # Только Vite dev server (без Rust backend)
```

### Сборка

```bash
npm run build              # Сборка frontend (проверка типов + Vite build)
npm run tauri build        # Полная сборка release-версии
```

### Тестирование

```bash
npm run test               # Vitest в режиме watch
npm run test:run           # Однократный запуск frontend тестов (24 теста)

cd src-tauri && cargo test # Запуск backend тестов (9 тестов)
```

### Прочее

```bash
npm run preview            # Предпросмотр production сборки frontend
```

---

## Архитектура

### Процессная модель

При запуске `npm run dev`:
1. **tauri dev** запускает два параллельных процесса:
   - Vite dev server (порт 1420) — frontend
   - Cargo run — Rust backend с tray и SQLite

### Компоненты системы

```
[Tray App]
    │
    ├── Agent (сборщик активности)
    │       ├── Канал 1: Время (активное приложение, idle)
    │       ├── Канал 2: Ввод (счётчики нажатий)
    │       └── Канал 3: Метаданные (настройки)
    │
    ├── Storage (SQLite)
    │
    └── Dashboard (Vue UI)
```

### Экраны UI

| Экран | Назначение |
|-------|------------|
| Сегодня | KPI за день, топ приложений, график интенсивности ввода |
| Неделя | Статистика по дням, топ приложений за неделю |
| Таймлайн | Детальная лента активности по времени |
| Настройки | Интервал опроса, порог idle, переключатели приватности |

---

## Сущности базы данных

| Таблица | Назначение |
|---------|------------|
| `app` | Приложения (exe_path, display_name) |
| `activity_sample` | Снимки активности (timestamp, app_id, is_idle, window_title) |
| `input_sample` | Агрегаты ввода (timestamp_bucket, keyboard_presses, mouse_clicks) |
| `settings` | Настройки приложения |

---

## Правила разработки

### Управление версией
Версия указывается **только в `package.json`**. Остальные файлы подхватывают автоматически:
- `src-tauri/Cargo.toml`: `version = { workspace = true }`
- `src-tauri/tauri.conf.json`: `"version": "../package.json"`

### Стиль кода
- Vue 3 Composition API (`<script setup lang="ts">`)
- CSS-переменные библиотеки dobruniaui-vue (`--dbru-*`)
- Класс `dbru-root` на корневом элементе для применения темы

### Single-instance
Приложение использует Windows Mutex (`Local\WorkaSingleInstanceMutex`) для защиты от запуска нескольких экземпляров.

### Поведение окна
- Закрытие окна (крестик) → сворачивание в трей
- Выход только через меню трея

---

## Windows API (используемые)

| API | Назначение |
|-----|------------|
| `GetForegroundWindow` | Получение активного окна |
| `GetWindowThreadProcessId` | Получение процесса по HWND |
| `GetWindowTextW` | Получение заголовка окна |
| `GetLastInputInfo` | Получение времени простоя |
| `CreateMutexW` | Single-instance защита |

---

## Документация проекта

| Файл | Описание |
|------|----------|
| `README.md` | Общее описание, быстрый старт, команды |
| `architecture.md` | Архитектура, компоненты, сущности БД |
| `stack.md` | Технологический стек и обоснование |
| `functional.md` | Требования и уровни фич |
| `RULES.md` | Правила для AI-помощника |

---

## Зависимости

### npm (package.json)

**Основные:**
- `vue` — UI фреймворк
- `@tauri-apps/api` — Tauri API для frontend
- `chart.js`, `vue-chartjs` — Графики
- `dobruniaui-vue` — UI компоненты

**Dev:**
- `vite` — Сборщик
- `typescript`, `vue-tsc` — Типизация
- `vitest`, `@testing-library/vue` — Тестирование
- `@tauri-apps/cli` — Tauri CLI

### Cargo (src-tauri/Cargo.toml)

- `tauri` — Core + tray + image-png features
- `tauri-plugin-opener` — Открытие ссылок
- `rusqlite` — SQLite (bundled)
- `windows` — Windows API
- `chrono` — Работа с датой/временем
- `serde`, `serde_json` — Сериализация

---

## Roadmap

### Уровень 1 — MVP (в разработке)
- [x] Работа в трее
- [x] Сбор данных: приложение, idle, заголовки окон
- [x] Счётчики ввода
- [x] Дашборд с 4 экранами
- [ ] Глобальные хуки для трекинга ввода

### Уровень 2 — Улучшения
- Категории приложений
- Цели/лимиты по времени
- Экспорт в CSV/JSON

### Уровень 3 — Продвинутые фичи
- Трекинг сайтов (расширение браузера)
- Фокус-режим / помодоро
- Синхронизация между устройствами
