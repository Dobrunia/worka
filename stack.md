# Технологический стек (Windows 10)

## Desktop shell
Tauri v2

Причины:
- Малый размер
- Хорошая интеграция с Windows
- Поддержка tray
- Возможность писать системную логику на Rust

---

## UI
- Vue 3
- Vite
- vitest
- TypeScript
- Chart.js - для графиков
- dobruniaui-vue - UI библиотека для Vue (используем для всего кроме графиков)
  - Документация: https://github.com/Dobrunia/DobruniaUI-vue

---

## Backend (внутри Tauri)
- Rust
- Windows API (через windows crate)
- SQLite (rusqlite)

---

## База данных
SQLite

Причины:
- Локальная
- Простая
- Быстрая
- Не требует сервера

---

## Tauri плагины
- `tauri-plugin-opener` — открытие ссылок

---

## Основные системные API (Windows)
- Получение активного окна (`GetForegroundWindow`)
- Получение процесса по HWND (`GetWindowThreadProcessId`)
- Получение заголовка окна (`GetWindowTextW`)
- Получение idle time (`GetLastInputInfo`)
- Single-instance mutex (`CreateMutexW`)
- Трей (через Tauri API)
- Автозапуск (через реестр или Tauri helper)

---

## Rust паттерны
- `Arc<AtomicBool>` — атомарный флаг остановки
- `Mutex<T>` — потокобезопасное состояние
- `std::thread::spawn` — фоновые задачи

---

## Тестирование
- **Frontend:** Vitest + @testing-library/vue (24 теста)
- **Backend:** Cargo test (9 тестов)
