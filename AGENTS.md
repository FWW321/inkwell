# Inkwell - AI-Powered Novel Writing App

Tauri v2 desktop app with React 19 frontend and Rust backend. Deeply integrated with AI for novel writing assistance.

## Tech Stack

- **Frontend**: React 19 + React Compiler + TypeScript 6 + Vite 8
- **Bundler**: Rolldown (Vite 8 unified Rust-based bundler, replaces esbuild + Rollup)
- **Linting**: Oxlint (Oxc, 50-100x faster than ESLint)
- **Formatting**: Oxfmt (Oxc, 30x faster than Prettier, Prettier-compatible)
- **Backend**: Rust (edition 2024) + Tauri v2
- **UI**: shadcn/ui v2 (baseui variant) + Tailwind CSS
- **Animations**: motion-plus (motion v4+)
- **AI Agent**: rig (Rust agent framework for building LLM-powered agents)
- **Package Manager**: pnpm
- **Dev Environment**: Nix flake (flake.nix)

## Project Structure

```
src/                  # React frontend
  App.tsx             # Main app component
  main.tsx            # Entry point
  styles.css          # Global styles
src-tauri/            # Rust backend
  src/
    main.rs           # Entry (windows_subsystem)
    lib.rs            # Tauri commands & plugin setup
  capabilities/       # Tauri permission configs
  tauri.conf.json     # Tauri app config
```

## Build Commands

```bash
# Frontend only
pnpm dev                        # Start Vite dev server (port 1420)
pnpm build                      # TypeScript check + Vite build

# Full Tauri app
pnpm tauri dev                  # Dev mode (frontend + Rust backend)
pnpm tauri build                # Production build

# Rust backend only (from src-tauri/)
cargo build                     # Build Rust code
cargo test                      # Run all Rust tests
cargo test test_name            # Run a single Rust test
cargo clippy                    # Lint Rust code
cargo fmt --check               # Check Rust formatting
cargo fmt                       # Auto-format Rust code

# Frontend linting/formatting/typecheck
pnpm exec oxlint                # Lint with Oxlint (50-100x faster than ESLint)
pnpm exec oxlint --fix          # Auto-fix lint issues
pnpm exec oxfmt                 # Format with Oxfmt (30x faster than Prettier)
pnpm exec oxfmt --check         # Check formatting without writing
pnpm exec tsc -b                # TypeScript type check (no emit)
```

## Code Style - TypeScript / React

### Imports
- Use path aliases configured in tsconfig/vite when available
- Group imports: react → third-party → @tauri-apps → local modules
- Use `import type` for type-only imports

### Formatting
- Double quotes for strings (matching existing code)
- 2-space indentation
- JSX in `.tsx` files only
- Prefer arrow functions for React components

### Types
- TypeScript strict mode is enabled (`"strict": true`)
- `noUnusedLocals` and `noUnusedParameters` are enforced
- Always provide explicit return types for exported functions
- Use `interface` for object shapes, `type` for unions/intersections
- Target: ES2024

### React Conventions
- React Compiler is active via `@rolldown/plugin-babel` + `reactCompilerPreset` (see vite.config.ts) — avoid unnecessary `useMemo`/`useCallback`/`React.memo`
- `@vitejs/plugin-react` v6 uses Oxc for React Refresh (no Babel dependency by default)
- Use `invoke<T>()` from `@tauri-apps/api/core` for Tauri IPC calls with explicit type params
- Keep components focused; extract hooks for complex logic
- Use functional components only (no class components)

### State Management
- Prefer React's built-in state (`useState`, `useReducer`) for local state
- Lift state up or use context for shared state; avoid premature global state libraries

## Code Style - Rust

### Formatting
- Run `cargo fmt` before committing — follow rustfmt defaults
- Run `cargo clippy` and fix all warnings

### Naming
- snake_case for functions, variables, modules
- PascalCase for types, structs, enums, traits
- `#[tauri::command]` functions use snake_case (JS side converts to camelCase automatically)

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `thiserror` for custom error types in library code
- Use `anyhow` for application-level error handling if needed
- Never panic in production code; use `.expect()` only in `main.rs` / test code
- Return proper error types from Tauri commands

### Tauri Commands
- All `#[tauri::command]` functions must be registered in `invoke_handler`
- Use strongly typed structs with `#[derive(Serialize, Deserialize)]` for IPC data
- Keep command handlers thin — delegate to service functions

## UI / Component Conventions

- Use **shadcn/ui v2** (baseui variant) for all UI components
  - Use the `shadcn` MCP tool to search, view, and add components
  - Use `shadcn get_item_examples_from_registries` for usage patterns
- Use **motion-plus** (Motion+ 付费订阅) for all animations — 可使用 Motion+ 专属功能和早期实验性功能
  - Use `motion-studio` MCP for spring/easing generation
  - Use `motion` skill for React integration guidance
  - Use `css-spring` skill for CSS spring animations
  - Use `see-transition` skill to visualize transitions
  - Use `motion-audit` skill to audit animation performance
- Use **ui-ux-pro-max** skill for UI/UX design decisions

## Git Conventions

- Use **conventional-git** skill for commit message format
- Format: `type(scope): description`
- Types: `feat`, `fix`, `refactor`, `docs`, `style`, `test`, `chore`, `perf`
- Use present tense, imperative mood ("add feature" not "added feature")

## Internationalization

- Chinese is the primary language for UI text and comments
- Use **humanizer-zh** skill to review/fix AI-sounding Chinese text
- Avoid stiff, formulaic phrasing in Chinese copy

## MCP Tools & Skills Reference

| Purpose | Tool / Skill |
|---------|-------------|
| UI components | `shadcn` MCP, `shadcn` skill |
| Animations | `motion-studio` MCP, `motion` skill |
| CSS springs | `css-spring` skill |
| Transition preview | `see-transition` skill |
| Animation audit | `motion-audit` skill |
| UI/UX design | `ui-ux-pro-max` skill |
| Tauri debugging | `mcp-server-tauri` MCP |
| Library docs query | `context7` MCP (`resolve-library-id` → `query-docs`) |
| Rust (通用) | `rust-router` skill |
| Rust (所有权/借用/生命周期) | `m01-ownership` skill |
| Rust (智能指针/资源管理) | `m02-resource` skill |
| Rust (可变性) | `m03-mutability` skill |
| Rust (泛型/trait/零成本抽象) | `m04-zero-cost` skill |
| Rust (类型驱动设计) | `m05-type-driven` skill |
| Rust (错误处理) | `m06-error-handling` skill |
| Rust (并发/异步) | `m07-concurrency` skill |
| Rust (领域建模) | `m09-domain` skill |
| Rust (性能优化) | `m10-performance` skill |
| Rust (crate 生态/依赖) | `m11-ecosystem` skill |
| Rust (资源生命周期) | `m12-lifecycle` skill |
| Rust (领域错误处理) | `m13-domain-error` skill |
| Rust (心智模型/学习) | `m14-mental-model` skill |
| Rust (反模式审查) | `m15-anti-pattern` skill |
| Rust (代码风格) | `coding-guidelines` skill |
| Rust (unsafe/FFI 审查) | `unsafe-checker` skill |
| Rust (函数调用图) | `rust-call-graph` skill |
| Rust (代码导航/LSP) | `rust-code-navigator` skill |
| Rust (新闻/动态) | `rust-daily` skill |
| Rust (依赖可视化) | `rust-deps-visualizer` skill |
| Rust (版本/crate 信息) | `rust-learner` skill |
| Rust (安全重构) | `rust-refactor-helper` skill |
| Rust (创建 crate skill) | `rust-skill-creator` skill |
| Rust (项目符号分析) | `rust-symbol-analyzer` skill |
| Rust (trait 实现探索) | `rust-trait-explorer` skill |
| Commit format | `conventional-git` skill |
| Chinese text | `humanizer-zh` skill |

## Key Architectural Notes

- Frontend runs on `localhost:1420` in dev; Tauri webview loads this URL
- Vite ignores `src-tauri/` for HMR (`watch.ignored` in vite.config.ts)
- Tauri capabilities define permissions in `src-tauri/capabilities/default.json`
- Rust lib name is `inkwell_lib` (Cargo.toml `name = "inkwell"` → `inkwell_lib::run()`)
- Window: 1280x800 default, min 900x600
- App identifier: `com.inkwell.app`
