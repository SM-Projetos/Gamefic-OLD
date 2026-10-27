# CHANGELOG — Gamefic

Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/), versionado por SemVer. Histórico completo de desenvolvimento nos commits `SNAPSHOT-xxxx` do `git log`.

## [Não lançado]

### Adicionado
- Detectores de família ampliados (Alpine/Void/Gentoo/Solus/NixOS), arquitetura real (uname) e catálogo com pacotes mapeados por família (`PkgNat`): `wine-cachyos` só é candidato no CachyOS/Arch.
- Fonte Snap Store distinta: só é ofertada com `snapd` funcionalmente em execução, com identificadores verificados na API oficial da Snap Store (steam e retroarch).
- Cache de detalhes com TTL, invalidado por qualquer ação de instalação/remoção, e exemplo `examples/dump_fontes.rs` para diagnóstico manual.
- Método de consulta de remotos Flatpak reais (`flatpak remotes`): disponibilidade concedida apenas por confirmação do remoto configurado, sem presumir flathub.
- Novos testes: mapeamento por família de catálogo, snap verificado, parser de remotos, idioma localizado e estados de consulta.

### Corrigido
- Consultas bem-sucedidas sem análise multi-idioma (`Versão:` PT-BR e `Version:`), evitando esconder fontes disponíveis do Flatpak em sistemas em português.
- Diferenciação de ausência confirmada, remoto ausente no escopo, falha de consulta (ex.: rede) e snapd parado; falhas temporárias mostram novo estado "indeterminado" na interface
- Interface da aba Apps: fontes não verificadas/viáveis não aparecem mais como opções no seletor; diagnóstico detalhado permanece na listagem de fontes.
- Listener de remoção de ferramentas de compatibilidade do Steam perdido no SNAPSHOT-0044 (botões Remover sem efeito).

## [0.1.0] — versão inicial (2026-10-22)

Primeira versão funcional do app: fundação visual, detecção real de sistema, monitoramento, histórico de eventos e análise de otimização.

### Fundação (SNAPSHOT-0001, 0002)

- Projeto Tauri 2 com frontend TypeScript vanilla (sem framework JS, sem Electron/GTK/Qt)
- Janela com 5 abas: Visão geral, Desempenho, Kernels, Otimização, Histórico e Diagnósticos
- Sistema de design centralizado (`src/styles/`): paleta, tipografia (Inter + JetBrains Mono com licenças OFL embutidas), espaçamento, componentes
- Temas: escuro e claro, mais modo auto (segue o sistema), com persistência em disco
- Painel de configurações com preferências JSON

### Detecção real do sistema (SNAPSHOT-0003)

- Snapshot real do sistema: distribuição, kernel, CPU (núcleos/threads/frequências), memória e swap, GPU e driver, Vulkan, sensores de temperatura
- Degradação honesta: dados indisponíveis retornam `null` e aparecem como "Não disponível" — nenhum valor fictício
- Exemplo `examples/dump_system.rs` para diagnóstico
- Estado de detecção exibido no topo da janela

### Monitoramento com gráficos (SNAPSHOT-0004)

- Coleta periódica em thread de fundo (histórico da sessão de 600 amostras)
- Intervalo configurável e persistido
- Uso de CPU (agregado), memória, temperatura, GPU (disponível quando o driver NVIDIA fornece via `nvidia-smi`)
- Gráficos em canvas na aba Desempenho, sem dependências externas no frontend
- Início e parada do monitoramento registrados como eventos

### Histórico de eventos e análise de otimização (SNAPSHOT-0005)

- Eventos persistidos em JSON com limite de 500 registros — sobrevivem ao fechamento do app
- Abras de filtro: por categoria (sistema, detecção, monitoramento, otimização) e por resultado (concluído, erro, aviso)
- Análise da aba Otimização: estados consultáveis do sistema (governador de CPU, Gamemode/MangoHud, Vulkan, swap) e recomendações justificadas pelos dados detectados, com classificação de risco (baixo/médio/alto)
- Perfis Equilibrado/Gamer/Economia de energia estruturados (0 ajustes implementáveis nesta versão — nenhuma ação sistêmica é executada; os textos informam isso)
- Limpeza do histórico com confirmação e registro do evento

### Build e acabamento (SNAPSHOT-0006)

- Build de produção: `.deb`, `.rpm` e AppImage
- Remoção do plugin `opener` (não usado) — permissões do webview restritas a `core:default`
- README com requisitos, comandos e estrutura

### Segurança e licenças

- Nenhuma execução genérica de shell: a detecção usa apenas binários fixos com argumentos fixos, e tudo é validado no Rust
- Sem operações com sudo/root; interface nunca é a fronteira de segurança
- Licença do projeto: GPL-3.0-only — todas as dependências são licenças livres (MIT/Apache-2.0/BSD/Zlib/MPL-2.0/Unicode/OFL), verificadas individualmente

### Limitações conhecidas

- Recomendações são apenas análise — nenhuma otimização é aplicada ao sistema nesta versão
- Métricas de GPU dependem do driver NVIDIA; em GPU sem suporte a `nvidia-smi`/AMD, campo aparece como "Não disponível"
- Testes de portabilidade limitados ao host de desenvolvimento (CachyOS, Wayland)
