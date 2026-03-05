# Repository Guidelines

## Project Structure & Module Organization
`worka` is a Tauri desktop app with a Vue 3 frontend.

- `src/`: frontend app code.
- `src/components/layout` and `src/components/ui`: reusable Vue components.
- `src/views`: page-level screens (`TodayView.vue`, `WeekView.vue`, etc.).
- `src/composables`: data hooks (`useTodayData.ts`, `useWeekData.ts`).
- `src/utils`: shared helpers.
- `src/tests/setup.ts`: Vitest global setup.
- `src-tauri/`: Rust backend, Tauri config, capabilities, and packaging assets.
- `public/`: static frontend assets.
- `dist/`: generated frontend build output.

## Build, Test, and Development Commands
- `npm install`: install JS dependencies.
- `npm run dev`: run full Tauri app in development (frontend + native shell).
- `npm run dev:vite`: run frontend only on Vite dev server.
- `npm run build`: type-check (`vue-tsc`) and build frontend.
- `npm run preview`: preview built frontend.
- `npm run test`: run Vitest in watch mode.
- `npm run test:run`: run tests once (CI-friendly).
- `npm run test:coverage`: generate text/json/html coverage reports.
- `cd src-tauri && cargo test`: run Rust backend tests.

## Coding Style & Naming Conventions
- Use TypeScript strict mode; keep code compatible with `tsconfig.json` (`strict`, no unused locals/params).
- Vue SFCs: PascalCase for component files (`KpiCard.vue`), composables as `useXxx.ts`, views as `XxxView.vue`.
- Prefer `@/` alias for imports from `src`.
- Use 2-space indentation in Vue/TS and keep modules focused and small.

## Testing Guidelines
- Frontend tests use Vitest + Testing Library with JSDOM.
- Place tests next to source files using `*.test.ts` (examples: `AppHeader.test.ts`, `SettingsView.test.ts`).
- Cover rendering, user interactions, and key state transitions.
- For backend changes, add/update Rust unit tests in `src-tauri/src`.

## Commit & Pull Request Guidelines
Current history mostly uses short messages (`fix`, `chore: init`). Prefer clearer Conventional Commit style going forward:
- `feat: add idle threshold validation`
- `fix: prevent duplicate tray menu event handlers`

PRs should include:
- concise summary and rationale,
- linked issue/task (if any),
- test evidence (`npm run test:run`, `cargo test`),
- screenshots/GIFs for UI changes (views/components).
