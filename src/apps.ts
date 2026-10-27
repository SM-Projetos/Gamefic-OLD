import { invoke } from "@tauri-apps/api/core";
import { mostrarPagina } from "./nav";

export interface FonteInfo {
  fonte: string;
  disponivel: boolean;
  instalado: boolean;
  versao?: string | null;
  versaoRepo?: string | null;
  motivo?: string | null;
  indeterminado?: boolean;
}

export interface AppDetail {
  id: string;
  nome: string;
  gui: boolean;
  fornecedor: string;
  licenca: string;
  descCurta: string;
  descLonga: string;
  icone?: string | null;
  fontes: FonteInfo[];
}

export interface InfoCompat {
  chave: string;
  nome: string;
  disponivel: boolean;
  versao_atual?: string | null;
  versao_repo?: string | null;
}

interface ForkList {
  wines: InfoCompat[];
  protons: InfoCompat[];
}

function esc(t: string): string {
  return t.replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c] as string),
  );
}

let detalheAtual: AppDetail | null = null;

function fonteLabel(f: string): string {
  if (f === "flatpak") return "Flatpak";
  if (f === "flatpak-user") return "Flatpak (usuário)";
  if (f.startsWith("snap:")) return `Snap Store (${f.split(":")[1]})`;
  if (f.startsWith("nativo:")) return `Nativo (${f.split(":")[1]})`;
  return f;
}

function setTopbarTitle(t: string): void {
  const el = document.getElementById("topbar-title");
  if (el) el.textContent = t;
}

function isMostarPagina(tab: string): void {
  mostrarPagina(tab);
  setTopbarTitle(TITLES[tab] ?? "Apps");
}

const TITLES: Record<string, string> = {
  "visao-geral": "Apps",
  "app-detalhe": "App",
};

export function irParaApps(): void {
  // aciona a transição de volta (zoom reverso) e limpa o sinal logo após a animação
  document.body.dataset.navVolta = "apps";
  isMostarPagina("visao-geral");
  setTopbarTitle("Apps");
  window.setTimeout(() => {
    delete document.body.dataset.navVolta;
  }, 300);
}

export function abrirApp(appId: string): void {
  invoke<AppDetail>("get_app_detail", { appId })
    .then((d) => {
      detalheAtual = d;
      renderDetail(d);
      mostrarPagina("app-detalhe");
      setTopbarTitle(d.nome);
    })
    .catch(() => alert("Falha ao carregar o aplicativo"));
}

function renderDetail(d: AppDetail): void {
  const card = document.getElementById("app-detalhe-card");
  if (!card) return;
  const icone = d.icone
    ? `<img src="${esc(d.icone)}" alt="">`
    : `<span style="font-size:40px">${esc(d.nome.charAt(0))}</span>`;

  const fontesMeta = d.fontes.map((f) => {
    const estado = f.instalado ? "Instalado" : f.disponivel ? "Disponível" : "Indisponível";
    const detalhes: string[] = [];
    if (f.instalado && f.versao) detalhes.push(`versão instalada: ${f.versao}`);
    if (f.versao && !f.instalado) detalhes.push(`versão no repositório: ${f.versao}`);
    if (f.motivo) detalhes.push(f.motivo);
    return (`
      <li class="kv-lista-item">${esc(fonteLabel(f.fonte))} — <strong>${estado}</strong>
      ${detalhes.length > 0 ? ` <span class="muted">(${esc(detalhes.join("; "))})</span>` : ""}
      ${!f.instalado && f.disponivel && f.versaoRepo ? ` <span class="muted num">${esc(f.versaoRepo)}</span>` : ""}</li>`);
  }).join("");

  const algumInstalado = d.fontes.some((f) => f.instalado);
  const fonteInstalada = d.fontes.find((f) => f.instalado);
  // §5: só fontes conferidas entram no seletor; estados de diagnóstico ficam separados
  const fontesOfertaveis = d.fontes.filter((f) => f.disponivel && !f.instalado);
  const algumIndeterminado = d.fontes.some((f) => f.indeterminado);
  const fontePadrao = algumInstalado ? fonteInstalada?.fonte ?? "" : fontesOfertaveis[0]?.fonte ?? "";

  // mensagem quando não há fonte instalável verificada (casos F e G)
  const semOpcoes = !algumInstalado && fontesOfertaveis.length === 0;
  const mensagemSemFonte = semOpcoes
    ? `
      <p class="field__status" id="app-fonte-status">${
        algumIndeterminado
          ? "Não foi possível verificar as fontes agora (falha temporária) — recarregue os detalhes para tentar novamente."
          : "Não encontrado em nenhuma fonte compatível com este sistema."
      }</p>`
    : "";

  card.innerHTML = `
    <div class="app-detalhe__icone" aria-hidden="true">${icone}</div>
    <div class="app-detalhe__head">
      <h2>${esc(d.nome)}</h2>
      <div class="app-detalhe__meta">
        <span class="pill pill--info">Fornecedor: ${esc(d.fornecedor)}</span>
        <span class="pill pill--info">Licença: ${esc(d.licenca)}</span>
      </div>
      <p class="app-detalhe__desc-breve">${esc(d.descCurta)}</p>
      <p class="app-detalhe__desc">${esc(d.descLonga)}</p>

      <ul class="kv-lista app-detalhe__fontes-lista">${fontesMeta}</ul>

      ${fontesOfertaveis.length > 1 ? `
      <div class="app-detalhe__fontes fonte-dropdown-wrapper">
        <button type="button" class="btn btn--primary fonte-dropdown-btn" id="fonte-dropdown-btn">
          <span id="fonte-dropdown-label">${fontePadrao ? fonteLabel(fontePadrao) : "Nativo"}</span>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
        </button>
        <div class="fonte-dropdown-menu" id="fonte-dropdown-menu" hidden>
          ${fontesOfertaveis.map((f) => `<button type="button" class="fonte-dropdown-menu__item" data-fonte="${esc(f.fonte)}">${esc(fonteLabel(f.fonte))}</button>`).join("")}
        </div>
        <p class="field__status app-fonte-status" id="app-fonte-status">&nbsp;</p>
      </div>
      ` : ""}
      ${mensagemSemFonte}

      <div class="app-detalhe__acoes">
        <button class="btn btn--primary" id="btn-app-instalar" type="button">Instalar</button>
        <button class="btn btn--ghost" id="btn-app-atualizar" type="button">Atualizar</button>
        <button class="btn btn--ghost" id="btn-app-remover" type="button">Remover</button>
        <span class="muted app-acao-status" id="app-acao-status">&nbsp;</span>
      </div>
    </div>
  `;
  const dropdownBtn = card.querySelector<HTMLButtonElement>("#fonte-dropdown-btn");
  const dropdownMenu = card.querySelector<HTMLDivElement>("#fonte-dropdown-menu");
  const dropdownLabel = card.querySelector<HTMLSpanElement>("#fonte-dropdown-label");
  fonteAtualGlobal = fontePadrao;
  let fonteAtual = fontePadrao;

  dropdownBtn?.addEventListener("click", () => {
    dropdownMenu?.toggleAttribute("hidden");
  });

  card.querySelectorAll<HTMLButtonElement>(".fonte-dropdown-menu__item").forEach((item) => {
    item.addEventListener("click", () => {
      fonteAtual = item.dataset.fonte ?? fontePadrao;
      fonteAtualGlobal = fonteAtual;
      if (dropdownLabel) dropdownLabel.textContent = fonteLabel(fonteAtual);
      dropdownMenu?.setAttribute("hidden", "");
      atualizarEstadoEntrada();
    });
  });

  atualizarEstadoEntrada();

  const instalar = card.querySelector<HTMLButtonElement>("#btn-app-instalar");
  const atualizar = card.querySelector<HTMLButtonElement>("#btn-app-atualizar");
  const remover = card.querySelector<HTMLButtonElement>("#btn-app-remover");
  instalar?.addEventListener("click", () => executarAcao("instalar"));
  atualizar?.addEventListener("click", () => executarAcao("atualizar"));
  remover?.addEventListener("click", () => executarAcao("remover"));
}

let fonteAtualGlobal: string = "";

function fonteEscolhida(): FonteInfo | null {
  if (!fonteAtualGlobal || !detalheAtual) return null;
  return detalheAtual.fontes.find((f) => f.fonte === fonteAtualGlobal) ?? null;
}

function atualizarEstadoEntrada(): void {
  const f = fonteEscolhida();
  const status = document.querySelector<HTMLElement>("#app-fonte-status");
  const btnInstalar = document.querySelector<HTMLButtonElement>("#btn-app-instalar");
  const btnAtualizar = document.querySelector<HTMLButtonElement>("#btn-app-atualizar");
  const btnRemover = document.querySelector<HTMLButtonElement>("#btn-app-remover");
  const dropdownWrapper = document.querySelector<HTMLElement>(".fonte-dropdown-wrapper");
  const btnAbrir = document.querySelector<HTMLButtonElement>("#btn-app-abrir");

  const algumInstalado = detalheAtual ? detalheAtual.fontes.some((x) => x.instalado) : false;
  const semOpcoes = detalheAtual
    ? !algumInstalado &&
      detalheAtual.fontes.filter((x) => x.disponivel && !x.instalado).length === 0
    : false;

  if (dropdownWrapper) dropdownWrapper.style.display = algumInstalado ? "none" : "inline-block";
  if (btnInstalar) btnInstalar.style.display = algumInstalado || semOpcoes ? "none" : "inline-flex";

  if (status) {
    status.textContent = f
      ? f.instalado
        ? "Já instalado nesta fonte"
        : f.disponivel
          ? "Disponível para instalar"
          : "Indisponível"
      : "—";
  }

  const fonteInstalada = detalheAtual?.fontes.find((x) => x.instalado);
  const temAtualizacao = fonteInstalada ? (fonteInstalada.versaoRepo && fonteInstalada.versaoRepo !== fonteInstalada.versao) : false;

  if (btnAtualizar) {
    btnAtualizar.style.display = (algumInstalado && temAtualizacao) ? "inline-flex" : "none";
    btnAtualizar.disabled = false;
    btnAtualizar.textContent = temAtualizacao ? "Atualizar" : "Atualizado";
  }

  if (btnRemover) {
    btnRemover.style.display = algumInstalado ? "inline-flex" : "none";
    btnRemover.disabled = !f || !f.instalado;
  }

  if (!btnAbrir && detalheAtual?.gui && algumInstalado && fonteInstalada) {
    const acoes = document.querySelector<HTMLDivElement>(".app-detalhe__acoes");
    if (acoes) {
      const btn = document.createElement("button");
      btn.className = "btn btn--primary";
      btn.id = "btn-app-abrir";
      btn.textContent = "Abrir";
      btn.type = "button";
      btn.addEventListener("click", () => {
        const id = detalheAtual?.id;
        if (!id) return;
        invoke<string>("abrir_app", { appId: id })
          .catch((e) => console.error("Falha ao abrir:", e));
      });
      acoes.insertBefore(btn, acoes.firstChild);
    }
  }
}

function revalidarEstados(): void {
  // avisa a listagem geral para recarregar o estado real (§3.3)
  window.dispatchEvent(new Event("estado-apps-mudou"));
}

function executarAcao(acao: "instalar" | "atualizar" | "remover"): void {
  const f = fonteEscolhida();
  const id = detalheAtual?.id;
  const status = document.querySelector<HTMLElement>("#app-acao-status");
  if (!f || !id || !status) return;
  status.textContent = acao === "instalar" ? "Instalando..." : acao === "atualizar" ? "Atualizando..." : "Removendo...";
  invoke<string>("app_action", { appId: id, acao, fonte: f.fonte })
    .then((msg) => {
      status.textContent = msg;
      if (detalheAtual?.id) abrirApp(detalheAtual.id);
      revalidarEstados();
    })
    .catch((e) => {
      status.textContent = `Erro: ${String(e)}`;
      revalidarEstados();
    });
}

// ---------- tiles clicáveis + card "+" ----------

export function wireAppClicks(): void {
  const ids = ["ov-launchers", "ov-tools", "ov-compat", "ov-emul"];
  ids.forEach((gid) => {
    const grid = document.getElementById(gid);
    if (!grid) return;
    grid.addEventListener("click", (ev) => {
      const target = ev.target as HTMLElement;
      const tile = target.closest<HTMLElement>(".tool-tile");
      if (!tile) return;
      if (tile.classList.contains("tool-tile--add")) {
        abrirModalCompat();
        return;
      }
      // itens de compatibilidade (Wine, Protons) não têm página de instalação
      if (gid === "ov-compat") return;
      const id = tile.dataset.appId;
      if (id && !id.startsWith("proton:")) abrirApp(id);
    });
  });
  document.getElementById("btn-voltar-apps")?.addEventListener("click", irParaApps);

// fechar modal de compatibilidade ao clicar fora da área do diálogo (backdrop do app)
  document.addEventListener("click", (ev) => {
    const modal = document.getElementById("modal-compat");
    if (!modal || modal.hidden) return;
    const target = ev.target as HTMLElement;
    // não interfere em cliques em elementos que abrem o modal
    if (target.closest(".tool-tile--add")) return;
    // fecha se o clique não for dentro do diálogo
    if (!target.closest(".modal")) {
      ocultarModalCompat();
    }
  });
}

// ---------- modal de compatibilidade ----------

let versoesCarregadas = false;
let pollProgresso: number | null = null;
let wineVersoes: string[] = [];

interface ProgressoItem {
  id: string;
  nome: string;
  desc: string;
  pct?: number | null;
  estado: string; // "andamento" | "pausado" | "cancelado" | "concluido" | "erro"
  fase?: string | null; // "aguardando" | "baixando" | "instalando" | ...
  kbFeitos?: number | null;
  kbTotal?: number | null;
  velocidade?: number | null;
}

const CONTROLES = {
  pausar: "Pausar",
  retomar: "Retomar",
  cancelar: "Cancelar",
} as const;

/// controles disponíveis pelo estado real da operação
function acoesDoEstado(p: ProgressoItem): ("pausar" | "retomar" | "cancelar")[] {
  const ativos: ("pausar" | "retomar" | "cancelar")[] = [];
  const proton = p.id.startsWith("proton:");
  const fork = p.id.split(":")[1] ?? "";
  if (p.estado === "andamento") {
    const fase = p.fase ?? "";
    if (fase.startsWith("baixando")) {
      ativos.push("pausar", "cancelar");
    } else if (fase.startsWith("aguardando")) {
      // antes da transação/extrair: ainda é possível cancelar com segurança
      ativos.push("cancelar");
    }
  } else if (p.estado === "pausado" && proton && fork !== "") {
    ativos.push("retomar", "cancelar");
  }
  return ativos;
}

/// operações extraídas do id de progresso (ex.: "proton:ge:GE-Proton11-11")
function alvoDoId(p: ProgressoItem): { fork: string; versao: string } {
  const partes = p.id.split(":");
  return { fork: partes[1] ?? "", versao: partes.slice(2).join(":") };
}

function abrirModalCompat(): void {
  const modal = document.getElementById("modal-compat");
  if (!modal) return;
  modal.hidden = false;
  if (!versoesCarregadas) {
    versoesCarregadas = true;
    document.getElementById("proton-fork")?.addEventListener("change", carregarVersoesProton);
    document.getElementById("wine-fork")?.addEventListener("change", atualizarVersoesWine);
    document.getElementById("btn-instalar-wine")?.addEventListener("click", instalarWine);
    document.getElementById("btn-instalar-proton")?.addEventListener("click", instalarProton);
    document.getElementById("btn-fechar-compat")?.addEventListener("click", fecharModalCompat);
    document.getElementById("modal-scrim")?.addEventListener("click", ocultarModalCompat);
    // controles de download usam delegação de eventos
    document.getElementById("modal-progresso")?.addEventListener("click", aoClicarProgresso);
    // remoção de ferramentas de compat do Steam (delegação no contêiner da lista)
    document.getElementById("modal-protons-lista")?.addEventListener("click", aoClicarProtons);
    document.getElementById("btn-limpar-hist")?.addEventListener("click", () => limparHistorico());
  }
  // lista de ferramentas do Steam (estado real, sem cache) + cache de forks
  atualizarProtonsSteam();
  invoke<ForkList & { versoes?: Record<string, string[]> }>("cache_compat_forks")
    .then((dados) => {
      forksCacheVersoes = dados.versoes ?? {};
      if ((dados.wines?.length ?? 0) > 0 || (dados.protons?.length ?? 0) > 0) {
        forksWines = dados.wines ?? [];
        forksProtons = dados.protons ?? [];
        renderForks();
        carregarVersoesProton();
        carregarVersoesWine();
      }
    })
    .catch(() => {});
  // atualização em segundo plano
  carregarForks();
  // estado dos processos em andamento (independente do ciclo da janela)
  iniciarPollProgresso();
}

function fecharModalCompat(): void {
  pararPollProgresso();
  document.getElementById("modal-compat")?.setAttribute("hidden", "");
  // o histórico persiste na sessão: fechamentos não limpam operações
}

function ocultarModalCompat(): void {
  document.getElementById("modal-compat")?.setAttribute("hidden", "");
}

// ---------- ciclo de vida dos processos (independente da janela) ----------

function atualizarProgresso(): void {
  invoke<ProgressoItem[]>("progresso_compat")
    .then(renderProgresso)
    .catch(() => {});
}

function iniciarPollProgresso(): void {
  if (pollProgresso !== null) return;
  atualizarProgresso();
  pollProgresso = window.setInterval(atualizarProgresso, 500);
}

function pararPollProgresso(): void {
  if (pollProgresso !== null) {
    window.clearInterval(pollProgresso);
    pollProgresso = null;
  }
}

/// operaões terminadas já sinalizadas para revalidação da aba Apps
let terminalSinalizado = new Set<string>();

function sinalizarTermino(lista: ProgressoItem[]): void {
  for (const p of lista) {
    if (
      (p.estado === "concluido" || p.estado === "erro" || p.estado === "cancelado") &&
      !terminalSinalizado.has(p.id)
    ) {
      terminalSinalizado.add(p.id);
      revalidarEstados();
    }
  }
}

function renderProgresso(lista: ProgressoItem[]): void {
  const area = document.getElementById("modal-progresso");
  if (!area) return;
  if (lista.length === 0) {
    area.hidden = true;
    area.innerHTML = "";
    atualizarBotoesHistorico([]);
    return;
  }
  area.hidden = false;
  area.innerHTML = lista
    .map((p) => {
      const concluido = p.estado === "concluido";
      const pct = concluido ? 100 : p.pct ?? null;
      const rotuloEstado: Record<string, string> = {
        andamento: pct !== null && p.pct !== null ? "Baixando" : "Em andamento",
        pausado: "Pausado",
        cancelando: "Cancelando...",
        cancelado: "Cancelado",
        concluido: "Concluído",
        erro: "Erro",
      };
      const rotPct = rotuloEstado[p.estado] ?? "—";
      const indeterminado =
        p.estado === "andamento" && pct === null && p.fase !== "instalando";
      const barra = p.fase === "instalando" && !concluido ? undefined : pct;
      const acoes = acoesDoEstado(p)
        .map((a) => {
          const { fork, versao } = alvoDoId(p);
          return `<button class="btn btn--outline btn--sm" data-acao="${a}" data-id="${esc(p.id)}" data-fork="${esc(fork)}" data-versao="${esc(versao)}" type="button">${CONTROLES[a]}</button>`;
        })
        .join("");
      const vel =
        p.velocidade && p.velocidade > 0
          ? ` · ${(p.velocidade / 1e6).toFixed(1)} MB/s`
          : "";
      return `
      <div class="compat-progresso-item" data-estado="${esc(p.estado)}">
        <div class="compat-progresso-head">
          <span class="compat-progresso-nome">${esc(p.nome)}</span>
          <span class="compat-progresso-estado" data-estado="${esc(p.estado)}">${esc(rotPct)}</span>
        </div>
        <div class="progress${indeterminado ? " progress--indeterminado" : ""}">
          <div class="progress__bar" style="width:${barra === undefined ? 35 : barra}%"></div>
        </div>
        <p class="muted compat-progresso-desc">${esc(p.desc)}${esc(vel)}</p>
        ${acoes ? `<div class="compat-progresso-acoes">${acoes}</div>` : ""}
      </div>`;
    })
    .join("");
  atualizarBotoesHistorico(lista);
  sinalizarTermino(lista);
}

/// botão único Limpar: remove somente operações em estado terminal
/// (concluído, falhou, cancelado); operações em andamento/pausadas ficam
function atualizarBotoesHistorico(lista: ProgressoItem[]): void {
  const area = document.getElementById("modal-historico-botoes");
  if (!area) return;
  const terminal = lista.some(
    (p) => p.estado === "concluido" || p.estado === "erro" || p.estado === "cancelado",
  );
  area.hidden = !terminal;
}

function limparHistorico(): void {
  invoke("limpar_historico_compat", { modo: "finalizados" })
    .then(() => atualizarProgresso())
    .catch(() => {});
}

/// controles do item de progresso: executam sobre a operação real
function aoClicarProgresso(ev: Event): void {
  const botao = (ev.target as HTMLElement).closest("button[data-acao]");
  if (!botao) return;
  const acao = botao.getAttribute("data-acao");
  const id = botao.getAttribute("data-id") ?? "";
  if (acao === "pausar") {
    invoke("pausar_compat", { id }).catch(() => {});
  } else if (acao === "cancelar") {
    invoke<string>("cancelar_compat", { id })
      .catch((e) => statusCompat(String(e)))
      .then(() => atualizarProgresso());
  } else if (acao === "retomar") {
    const fork = botao.getAttribute("data-fork") ?? "";
    const versao = botao.getAttribute("data-versao") ?? "";
    invoke("retomar_compat", { fork, versao })
      .then(() => atualizarProgresso())
      .catch((e) => statusCompat(`Falha ao retomar: ${String(e)}`));
  }
}

let forksWines: InfoCompat[] = [];
let forksProtons: InfoCompat[] = [];
let forksCacheVersoes: Record<string, string[]> = {};

function renderForks(): void {
  const sel = document.getElementById("wine-fork") as HTMLSelectElement | null;
  if (sel && forksWines.length > 0) {
    const anterior = sel.value;
    sel.innerHTML = forksWines
      .map((w) => `<option value="${esc(w.chave)}">${esc(w.nome)}${w.disponivel ? "" : " — indisponível"}</option>`)
      .join("");
    sel.value = forksWines.some((w) => w.chave === anterior) ? anterior : sel.value;
    atualizarVersoesWine();
    atualizarInfoWine();
  }
  const p = document.getElementById("proton-fork") as HTMLSelectElement | null;
  if (p && forksProtons.length > 0) {
    const anterior = p.value;
    p.innerHTML = forksProtons
      .map((w) => `<option value="${esc(w.chave)}">${esc(w.nome)}${w.disponivel ? "" : " — indisponível"}</option>`)
      .join("");
    if (anterior) {
      p.value = forksProtons.some((w) => w.chave === anterior) ? anterior : p.value;
    }
  }
}

function carregarForks(): void {
  invoke<ForkList>("listar_compat_forks")
    .then((dados) => {
      forksWines = dados.wines ?? [];
      forksProtons = dados.protons ?? [];
      renderForks();
      const p = document.getElementById("proton-fork") as HTMLSelectElement | null;
      if (p && !forksCacheVersoes[p.value]) carregarVersoesProton();
    })
    .catch((e) => {
      const info = document.getElementById("wine-info");
      if (info) info.textContent = `Falha ao carregar forks: ${String(e)}`;
    });
}

function atualizarVersoesWine(): void {
  const sel = document.getElementById("wine-fork") as HTMLSelectElement | null;
  const v = document.getElementById("wine-versao") as HTMLSelectElement | null;
  if (!sel || !v) return;
  const f = forksWines.find((w) => w.chave === sel.value);
  const rot = f?.versao_repo ? `latest (${f.versao_repo})` : "latest";
  const opcoes = wineVersoes
    .map((ver) => {
      const rotulo = `${ver}${ver.endsWith(".0") ? " (estável)" : " (desenvolvimento)"}`;
      return `<option value="${esc(ver)}">${esc(rotulo)}</option>`;
    })
    .join("");
  v.innerHTML = `<option value="latest">${esc(rot)}, versão oficial do WineHQ</option>${opcoes}`;
}

/// versões do Wine: cache em disco primeiro, depois a fonte oficial (GitLab do WineHQ)
function carregarVersoesWine(): void {
  const emCache = forksCacheVersoes["wine-oficial"];
  if (emCache && emCache.length > 0) {
    wineVersoes = emCache;
    atualizarVersoesWine();
  }
  invoke<string[]>("listar_wine_versoes")
    .then((l) => {
      if (l && l.length > 0) {
        wineVersoes = l;
        forksCacheVersoes["wine-oficial"] = l;
        atualizarVersoesWine();
      }
    })
    .catch(() => {});
}

let wineEmModoRemocao = false;
function atualizarInfoWine(): void {
  const sel = document.getElementById("wine-fork") as HTMLSelectElement | null;
  const info = document.getElementById("wine-info");
  const botao = document.getElementById("btn-instalar-wine") as HTMLButtonElement | null;
  if (!sel || !info) return;
  const f = forksWines.find((w) => w.chave === sel.value);
  // ferramenta instalada → somente Remover (§5.2)
  const instalado = Boolean(f?.versao_atual);
  wineEmModoRemocao = instalado;
  if (botao) botao.textContent = instalado ? "Remover" : "Instalar";
  info.textContent = f
    ? instalado
      ? `Já instalado (${f.versao_atual}) — desinstale antes de instalar outra variante`
      : f.versao_repo
        ? `Disponível no repositório: ${f.versao_repo}`
        : "Não disponível nos repositórios"
    : "—";
}

function carregarVersoesProton(): void {
  const sel = document.getElementById("proton-fork") as HTMLSelectElement | null;
  const v = document.getElementById("proton-versao") as HTMLSelectElement | null;
  const info = document.getElementById("proton-info");
  if (!sel || !v || !info) return;
  const f = forksProtons.find((w) => w.chave === sel.value);
  v.innerHTML = `<option value="latest">latest</option>`;
  info.textContent = f ? `Procurando versões...` : "—";
  if (!f) return;
  if (f.chave === "steam") {
    info.textContent = "Gerenciado pelo próprio Steam (só leitura, não baixável)";
    return;
  }
  const emCache = forksCacheVersoes[sel.value];
  if (emCache && emCache.length > 0) {
    const resto = emCache.map((ver) => {
      const rotulo = ver.length > 46 ? `${ver.slice(0, 46)}…` : ver;
      return `<option value="${esc(ver)}">${esc(rotulo)}</option>`;
    }).join("");
    v.innerHTML = `<option value="latest">latest</option>${resto}`;
    info.textContent = `${emCache.length} versões (cache)`;
  }
  invoke<string[]>("listar_proton_versoes", { fork: sel.value })
    .then((l) => {
      forksCacheVersoes[sel.value] = l ?? [];
      const resto = (l ?? []).map((ver) => {
        const rotulo = ver.length > 46 ? `${ver.slice(0, 46)}…` : ver;
        return `<option value="${esc(ver)}">${esc(rotulo)}</option>`;
      }).join("");
      v.innerHTML = `<option value="latest">latest</option>${resto}`;
      if (f.versao_atual) {
        info.textContent = "Já há uma versão deste fork instalada";
      } else {
        info.textContent = l && l.length > 0 ? `${l.length} versões disponíveis` : "Sem versões";
      }
    })
    .catch((e) => {
      if (!emCache) info.textContent = `Falha nas versões: ${String(e)}`;
    });
}

function statusCompat(msg: string): void {
  const el = document.getElementById("compat-status");
  if (el) {
    el.hidden = false;
    el.textContent = msg;
  }
}

// ferramentas instaladas do Steam são listadas na aba Apps (§5.1);
// o histórico do modal contém apenas operações iniciadas pelo Gamefic

function instalarWine(): void {
  const sel = document.getElementById("wine-fork") as HTMLSelectElement | null;
  if (!sel) return;
  if (wineEmModoRemocao) {
    if (!window.confirm(`Remover ${sel.value}? desinstala a variante pelo gerenciador de pacotes`)) {
      return;
    }
    statusCompat(`Removendo Wine (${sel.value})... — autorize no pkexec`);
    invoke<string>("remover_wine_compat", { pkg: sel.value })
      .then((msg) => {
        statusCompat(msg ?? "Removido");
        carregarForks();
      })
      .catch((e) => statusCompat(`Erro: ${String(e)}`));
    return;
  }
  statusCompat("Instalando Wine (pacote " + sel.value + ")... — autorize no pkexec");
  invoke<string>("instalar_wine", { pkg: sel.value })
    .then(() => {
      statusCompat("Instalando em segundo plano — pode fechar a janela com segurança");
      atualizarProgresso();
    })
    .catch((e) => statusCompat(`Erro: ${String(e)}`));
}

function instalarProton(): void {
  const fork = document.getElementById("proton-fork") as HTMLSelectElement | null;
  const versao = document.getElementById("proton-versao") as HTMLSelectElement | null;
  if (!fork || !versao) return;
  statusCompat(`Baixando ${fork.value} ${versao.value}... (poderá demorar)`);
  invoke<string>("instalar_proton_versao", { fork: fork.value, versao: versao.value })
    .then(() => {
      statusCompat("Baixando em segundo plano — pode fechar a janela com segurança");
      atualizarProgresso();
    })
    .catch((e) => statusCompat(`Erro: ${String(e)}`));
}

// ---------- ferramentas instaladas do Steam (oficiais + customs) ----------

interface ProtonLocal {
  nome: string;
  versao?: string | null;
  caminho: string;
  biblioteca: string;
  tipo: string; // "oficial" | "custom"
}

function atualizarProtonsSteam(): void {
  const area = document.getElementById("modal-protons-lista");
  if (!area) return;
  invoke<ProtonLocal[]>("listar_protons_steam")
    .then((lista) => {
      if (!lista || lista.length === 0) {
        area.hidden = true;
        area.innerHTML = "";
        return;
      }
      area.hidden = false;
      area.innerHTML = `<h4>Ferramentas de compatibilidade do Steam</h4>` +
        lista
          .map((p) => {
            const rotulo = p.versao ? `${p.nome} — ${p.versao}` : p.nome;
            return `<div class="compat-progresso-item" data-caminho="${esc(p.caminho)}">
              <div class="compat-progresso-head">
                <span class="compat-progresso-nome">${esc(rotulo)}</span>
                <span class="compat-progresso-estado" data-estado="instalado">${esc(p.tipo)}</span>
              </div>
              <p class="muted compat-progresso-desc">${esc(p.caminho)}</p>
              <div class="compat-progresso-acoes">
                <button class="btn btn--outline btn--sm" data-remover="${esc(p.caminho)}" type="button">Remover</button>
              </div>
            </div>`;
          })
          .join("");
    })
    .catch(() => {
      area.hidden = true;
    });
}

/// remoção individual com confirmação; o backend valida identidade e local
function aoClicarProtons(ev: Event): void {
  const botao = (ev.target as HTMLElement).closest("button[data-remover]");
  if (!botao) return;
  const caminho = botao.getAttribute("data-remover") ?? "";
  const nome = caminho.split("/").pop() ?? caminho;
  if (!window.confirm(`Remover ${nome}? Esta ação não pode ser desfeita.`)) return;
  invoke<string>("remover_ferramenta_compat", { caminho })
    .then((msg) => {
      statusCompat(msg ?? "Removido");
      atualizarProtonsSteam();
      carregarForks();
    })
    .catch((e) => statusCompat(`Erro ao remover: ${String(e)}`));
}
