import { invoke } from "@tauri-apps/api/core";
import { mostrarPagina } from "./nav";

export interface FonteInfo {
  fonte: string;
  disponivel: boolean;
  instalado: boolean;
  versao?: string | null;
}

export interface AppDetail {
  id: string;
  nome: string;
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
  isMostarPagina("visao-geral");
  setTopbarTitle("Apps");
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

  const fonteOptions = d.fontes
    .map((f) => `<option value="${esc(f.fonte)}">${esc(fonteLabel(f.fonte))}${f.disponivel ? "" : " — indisponível"}</option>`)
    .join("");

  const fontesMeta = d.fontes.map((f) => {
    const estado = f.instalado ? "Instalado" : f.disponivel ? "Disponível" : "Indisponível";
    return (`
      <li class="kv-lista-item">${esc(fonteLabel(f.fonte))} — <strong>${estado}</strong>
      ${f.versao ? ` <span class="muted num">${esc(f.versao)}</span>` : ""}</li>`);
  }).join("");

  const algumInstalado = d.fontes.some((f) => f.instalado);
  const fonteInstalada = d.fontes.find((f) => f.instalado);
  const fontePadrao = algumInstalado ? fonteInstalada?.fonte ?? d.fontes[0]?.fonte ?? "" : primeiraLivres?.fonte ?? d.fontes[0]?.fonte ?? "";

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

      <div class="app-detalhe__fontes fonte-dropdown-wrapper">
        <button type="button" class="btn btn--primary fonte-dropdown-btn" id="fonte-dropdown-btn">
          <span id="fonte-dropdown-label">${fontePadrao ? fonteLabel(fontePadrao) : "Nativo"}</span>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 9 6 6 6-6"/></svg>
        </button>
        <div class="fonte-dropdown-menu" id="fonte-dropdown-menu" hidden>
          ${d.fontes.map((f) => `<button type="button" class="fonte-dropdown-menu__item" data-fonte="${esc(f.fonte)}">${esc(fonteLabel(f.fonte))}${f.disponivel ? "" : " — indisponível"}</button>`).join("")}
        </div>
        <p class="field__status app-fonte-status" id="app-fonte-status">&nbsp;</p>
      </div>

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

  if (dropdownWrapper) dropdownWrapper.style.display = algumInstalado ? "none" : "inline-block";
  if (btnInstalar) btnInstalar.style.display = algumInstalado ? "none" : "inline-flex";

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
  const temAtualizacao = fonteInstalada ? (fonteInstalada.versao_repo && fonteInstalada.versao_repo !== fonteInstalada.versao) : false;

  if (btnAtualizar) {
    btnAtualizar.style.display = algumInstalado ? "inline-flex" : "none";
    btnAtualizar.disabled = !f || !f.instalado || !temAtualizacao;
    btnAtualizar.textContent = temAtualizacao ? "Atualizar" : "Atualizado";
  }

  if (btnRemover) {
    btnRemover.style.display = algumInstalado ? "inline-flex" : "none";
    btnRemover.disabled = !f || !f.instalado;
  }

  if (!btnAbrir && algumInstalado && fonteInstalada) {
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

function btnResumo(b: HTMLButtonElement | null): HTMLButtonElement | null {
  return b;
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
    })
    .catch((e) => {
      status.textContent = `Erro: ${String(e)}`;
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
  estado: string; // "andamento" | "concluido" | "erro"
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
  }
  // preenche imediatamente a partir do cache em disco
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
  renderProgresso([]);
  invoke("limpar_progresso_compat").catch(() => {});
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

function renderProgresso(lista: ProgressoItem[]): void {
  const area = document.getElementById("modal-progresso");
  if (!area) return;
  if (lista.length === 0) {
    area.hidden = true;
    area.innerHTML = "";
    return;
  }
  area.hidden = false;
  area.innerHTML = lista
    .map((p) => {
      const pct =
        p.estado === "concluido" ? 100 : p.estado === "erro" ? 0 : p.pct ?? null;
      const rotPct =
        p.estado === "concluido"
          ? "Concluído"
          : p.estado === "erro"
            ? "Erro"
            : pct !== null
              ? `${pct}%`
              : "—";
      const indeterminado = p.estado === "andamento" && pct === null;
      return `
      <div class="compat-progresso-item" data-estado="${esc(p.estado)}">
        <div class="compat-progresso-head">
          <span class="compat-progresso-nome">${esc(p.nome)}</span>
          <span class="compat-progresso-estado" data-estado="${esc(p.estado)}">${esc(rotPct)}</span>
        </div>
        <div class="progress${indeterminado ? " progress--indeterminado" : ""}">
          <div class="progress__bar" style="width:${(pct ?? 35) + "%"}"></div>
        </div>
        <p class="muted compat-progresso-desc">${esc(p.desc)}</p>
      </div>`;
    })
    .join("");
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

function atualizarInfoWine(): void {
  const sel = document.getElementById("wine-fork") as HTMLSelectElement | null;
  const info = document.getElementById("wine-info");
  if (!sel || !info) return;
  const f = forksWines.find((w) => w.chave === sel.value);
  info.textContent = f
    ? f.versao_atual
      ? `Já instalado (${f.versao_atual})`
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

function instalarWine(): void {
  const sel = document.getElementById("wine-fork") as HTMLSelectElement | null;
  if (!sel) return;
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
