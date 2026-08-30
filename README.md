# Sentience

App de bandeja (Tauri 2 + React + TypeScript) que mostra a webcam no centro da janela e a emoção detectada abaixo do preview. A emoção ainda é um placeholder (`Neutro`); o modelo em Rust entra depois.

A janela abre pelo ícone da bandeja (320×340, sem borda, always-on-top) e some ao perder o foco.

## Stack

- Frontend: React 19, Vite 7, Tailwind CSS 4, Bun
- Desktop: Tauri 2 (Rust)

## Pré-requisitos

- [Bun](https://bun.sh)
- [Rust](https://www.rust-lang.org/tools/install)
- Câmera disponível (o app pede permissão via `getUserMedia`)

## Desenvolvimento

```bash
bun install
bun run tauri dev
```

Clique no ícone da bandeja para abrir a janela.

## Build

```bash
bun run tauri build
```
