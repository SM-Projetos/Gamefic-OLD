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

  const primeiraLivres = d.fontes.find((f) => f.disponivel);
  const fonteDefault = primeiraLivres?.fonte ?? d.fontes[0]?.fonte ?? "";

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

      <div class="app-detalhe__fontes">
        <label class="field"><span class="field__label">Fonte</span>
          <select class="field__control" id="app-fonte-selecao">${fonteOptions}</select>
        </label>
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
  const selectEl = card.querySelector<HTMLSelectElement>("#app-fonte-selecao");
  if (selectEl) {
    selectEl.value = fonteDefault;
    selectEl.addEventListener("change", () => atualizarEstadoEntrada());
  }
  atualizarEstadoEntrada();

  const instalar = card.querySelector<HTMLButtonElement>("#btn-app-instalar");
  const atualizar = card.querySelector<HTMLButtonElement>("#btn-app-atualizar");
  const remover = card.querySelector<HTMLButtonElement>("#btn-app-remover");
  instalar?.addEventListener("click", () => executarAcao("instalar"));
  atualizar?.addEventListener("click", () => executarAcao("atualizar"));
  remover?.addEventListener("click", () => executarAcao("remover"));
}

function fonteEscolhida(): FonteInfo | null {
  const sel = document.querySelector<HTMLSelectElement>("#app-fonte-selecao");
  if (!sel || !detalheAtual) return null;
  return detalheAtual.fontes.find((f) => f.fonte === sel.value) ?? null;
}

function atualizarEstadoEntrada(): void {
  const f = fonteEscolhida();
  const status = document.querySelector<HTMLElement>("#app-fonte-status");
  const btnInstalar = document.querySelector<HTMLButtonElement>("#btn-app-instalar");
  const btnAtualizar = document.querySelector<HTMLButtonElement>("#btn-app-atualizar");
  const btnRemover = document.querySelector<HTMLButtonElement>("#btn-app-remover");
  if (status) {
    status.textContent = f
      ? f.instalado
        ? "Já instalado nesta fonte"
        : f.disponivel
          ? "Disponível para instalar"
          : "Indisponível"
      : "—";
  }
  if (btnResumo(btnInstalar)) btnResumo(btnInstalar)!.disabled = !f || !f.disponivel;
  if (btnResumo(btnAtualizar)) btnResumo(btnAtualizar)!.disabled = !f || !f.instalado;
  if (btnResumo(btnRemover)) btnResumo(btnRemover)!.disabled = !f || !f.instalado;
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
}

// ---------- modal de compatibilidade ----------

let versoesCarregadas = false;

function abrirModalCompat(): void {
  const modal = document.getElementById("modal-compat");
  if (!modal) return;
  modal.hidden = false;
  const carregado = versoesCarregadas;
  if (!versoesCarregadas) {
    versoesCarregadas = true;
    document.getElementById("proton-fork")?.addEventListener("change", carregarVersoesProton);
    document.getElementById("wine-fork")?.addEventListener("change", atualizarVersoesWine);
    document.getElementById("btn-instalar-wine")?.addEventListener("click", instalarWine);
    document.getElementById("btn-instalar-proton")?.addEventListener("click", instalarProton);
    document.getElementById("btn-fechar-compat")?.addEventListener("click", fecharModalCompat);
    document.getElementById("modal-scrim")?.addEventListener("click", fecharModalCompat);
  }
  if (!carregado) {
    // preenche imediatamente a partir do cache em disco
    invoke<ForkList & { versoes?: Record<string, string[]> }>("cache_compat_forks")
      .then((dados) => {
        forksCacheVersoes = dados.versoes ?? {};
        if ((dados.wines?.length ?? 0) > 0 || (dados.protons?.length ?? 0) > 0) {
          forksWines = dados.wines ?? [];
          forksProtons = dados.protons ?? [];
          renderForks();
          carregarVersoesProton();
        }
      })
      .catch(() => {});
    // atualização em segundo plano
    carregarForks();
  }
}

function fecharModalCompat(): void {
  document.getElementById("modal-compat")?.setAttribute("hidden", "");
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
  v.innerHTML = `<option value="latest">${esc(rot)}</option>`;
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
    .then((msg) => statusCompat(`OK: ${msg}`))
    .catch((e) => statusCompat(`Erro: ${String(e)}`));
}

function instalarProton(): void {
  const fork = document.getElementById("proton-fork") as HTMLSelectElement | null;
  const versao = document.getElementById("proton-versao") as HTMLSelectElement | null;
  if (!fork || !versao) return;
  statusCompat(`Baixando ${fork.value} ${versao.value}... (poderá demorar)`);
  invoke<string>("instalar_proton_versao", { fork: fork.value, versao: versao.value })
    .then((msg) => statusCompat(`OK: ${msg}`))
    .catch((e) => statusCompat(`Erro: ${String(e)}`));
}
