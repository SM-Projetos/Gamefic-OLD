# Gamefic

Aplicativo desktop para Linux voltado à otimização do computador para jogos: detecta hardware e sistema, monitora desempenho, consulta kernels/drivers e organiza ajustes com histórico de eventos.

- **Licença:** GPL-3.0-only (`LICENSE`)
- **Stack:** Tauri 2 · Rust · TypeScript · HTML/CSS

## Requisitos de desenvolvimento (Arch/CachyOS)

- Node.js + npm
- Rust estável (rustup)
- Bibliotecas dev: `gtk3`, `webkit2gtk-4.1`, `libudev` (com o toolchain base da distro)

## Comandos

```sh
npm install          # dependências do frontend
npm run tauri dev    # desenvolvimento (Vite + janela do app)
npm run tauri build  # empacotamento (.deb, .rpm, AppImage)
```

Verificações pontuais:

```sh
npx tsc --noEmit                  # tipos do frontend
(cd src-tauri && cargo check)     # backend
(cd src-tauri && cargo run --example dump_system)  # imprime o snapshot de detecção
```

## Estrutura

- `src/` — frontend (TypeScript, tokens de design, componentes, gráficos)
- `src-tauri/src/system/` — detecção real (distro, kernel, CPU, memória, GPU, Vulkan, sensores, ferramentas)
- `src-tauri/src/monitor.rs` — coleta periódica em thread de fundo com histórico da sessão
- `src-tauri/src/events.rs` — histórico persistido de eventos
- `src-tauri/src/recommendations.rs` — análise e recomendações justificadas pelos dados detectados

Dados indisponíveis são exibidos como "Não disponível" — nenhum valor fictício é apresentado como medição real.
