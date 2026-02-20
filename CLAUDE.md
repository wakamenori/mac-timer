# Development

```bash
npm run tauri:dev    # 開発サーバー起動
npm run tauri:build  # プロダクションビルド
```

# Static Analysis

Run all of the following after code changes:

```bash
# TypeScript
npx tsc --noEmit          # 型チェック
npm run lint              # ESLint (eslint src/ --max-warnings 0)

# Rust
cargo check --manifest-path src-tauri/Cargo.toml    # コンパイルチェック
cargo clippy --manifest-path src-tauri/Cargo.toml    # Lint
```
