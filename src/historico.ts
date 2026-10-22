import { invoke } from "@tauri-apps/api/core";
import type { SystemSnapshot } from "./system";
import { INDISPONIVEL } from "./system";

export interface Recommendation {
  id: string;
  title: string;
  rationale: string;
  expectedBenefit: string;
  limitNote: string;
  risk: "baixo" | "medio" | "alto";
  actionAvailable: boolean;
}

export interface SystemResource {
  id: string;
  name: string;
  state: string;
  detail?: string | null;
}

export interface ProfileScope {
  id: string;
  name: string;
  description: string;
  plannedAdjustments: string[];
  implementedAdjustments: number;
}

export interface Analysis {
  resources: SystemResource[];
  recommendations: Recommendation[];
  profiles: ProfileScope[];
}

export interface AppEvent {
  id: number;
  timestampMs: number;
  category:
    | "sistema"
    | "deteccao"
    | "configuracao"
    | "otimizacao"
    | "diagnostico";
  operation: string;
  result: "concluido" | "falha" | "cancelado";
  message: string;
  details?: string | null;
}

export function getAnalysis(): Promise<Analysis> {
  return invoke<Analysis>("get_analysis");
}

export function getEvents(
  filterCategory?: string,
  filterResult?: string,
): Promise<AppEvent[]> {
  return invoke<AppEvent[]>("get_events", {
    filterCategory: filterCategory || null,
    filterResult: filterResult || null,
  });
}

export function clearEvents(): Promise<number> {
  return invoke<number>("clear_events");
}

export function recordEvent(category: string, operation: string, result: string, message: string): void {
  invoke("add_event", { category, operation, result, message, details: null }).catch((e) =>
    console.warn("Falha ao registrar evento:", e),
  );
}

export function renderOtimizacao(analysis: Analysis): void {
  const recursos = document.getElementById("opt-recursos");
  if (recursos) {
    recursos.innerHTML = "";
    analysis.resources.forEach((r) => {
      const card = document.createElement("div");
      card.className = "card";
      card.innerHTML = `
        <div class="card-head"><span class="card__title">${esc(r.name)}</span></div>
        <p><span class="pill pill--info">${esc(r.state)}</span></p>
        ${r.detail ? `<p class="muted" style="margin-top: var(--space-2);">${esc(r.detail)}</p>` : ""}
      `;
      recursos.appendChild(card);
    });
    if (analysis.resources.length === 0) {
      recursos.appendChild(mensagemVazia("Nenhum estado consultável — recursos indisponíveis neste sistema"));
    }
  }

  const recomendacoes = document.getElementById("opt-recomendacoes");
  if (recomendacoes) {
    recomendacoes.innerHTML = "";
    analysis.recommendations.forEach((rec) => {
      const card = document.createElement("div");
      card.className = "card card--recommendation";
      const risco =
        rec.risk === "baixo"
          ? '<span class="pill pill--ok">Risco baixo</span>'
          : rec.risk === "medio"
            ? '<span class="pill pill--warn">Risco médio</span>'
            : '<span class="pill pill--error">Risco alto</span>';

      const acao = rec.actionAvailable
        ? '<button class="btn btn--primary btn--small">Aplicar</button>'
        : '<span class="pill pill--info">Análise apenas — ação em etapa futura</span>';

      card.innerHTML = `
        <div class="card-head"><span class="card__title">${esc(rec.title)}</span>${risco}</div>
        <p>${esc(rec.rationale)}</p>
        <dl class="kv" style="margin-top: var(--space-3);">
          <dt>Benefício esperado</dt><dd>${esc(rec.expectedBenefit)}</dd>
          <dt>Limitações</dt><dd>${esc(rec.limitNote)}</dd>
        </dl>
        <div style="margin-top: var(--space-3);">${acao}</div>
      `;
      recomendacoes.appendChild(card);
    });
    if (analysis.recommendations.length === 0) {
      recomendacoes.appendChild(mensagemVazia("Nenhuma recomendação aplicável aos dados detectados"));
    }
  }

  const perfis = document.getElementById("opt-perfis");
  if (perfis) {
    perfis.innerHTML = "";
    analysis.profiles.forEach((prof) => {
      const card = document.createElement("div");
      card.className = "card";
      const itens = prof.plannedAdjustments
        .map((a) => `<li>${esc(a)}</li>`)
        .join("");
      card.innerHTML = `
        <div class="card-head"><span class="card__title">${esc(prof.name)}</span><span class="pill pill--warn">Estrutura</span></div>
        <p>${esc(prof.description)}</p>
        <ul style="margin-top: var(--space-2); padding-left: var(--space-5); list-style: disc;">${itens}</ul>
        <p class="muted" style="margin-top: var(--space-2);">${prof.implementedAdjustments} de ${prof.plannedAdjustments.length} ajustes implementáveis nesta versão</p>
      `;
      perfis.appendChild(card);
    });
  }
}

export async function renderHistorico(): Promise<void> {
  const filtroCategoria = (document.getElementById("filtro-categoria") as HTMLSelectElement | null)?.value;
  const filtroResultado = (document.getElementById("filtro-resultado") as HTMLSelectElement | null)?.value;

  const lista = document.getElementById("lista-eventos");
  if (!lista) return;

  try {
    const events = await getEvents(filtroCategoria, filtroResultado);
    lista.innerHTML = "";

    if (events.length === 0) {
      lista.appendChild(mensagemVazia("Nenhum evento registrado com os filtros atuais"));
      return;
    }

    events.forEach((ev) => {
      const li = document.createElement("li");
      const data = new Date(ev.timestampMs);
      const quando = data.toLocaleString("pt-BR", { hour12: false });
      const resultado =
        ev.result === "concluido"
          ? '<span class="pill pill--ok">Concluído</span>'
          : ev.result === "falha"
            ? '<span class="pill pill--error">Falha</span>'
            : '<span class="pill pill--warn">Cancelado</span>';

      li.innerHTML = `
        <div style="display:flex; align-items:center; gap: var(--space-3);">
          <span class="pill pill--info">${esc(rotularCategoria(ev.category))}</span>
          ${resultado}
          <span class="muted num" style="margin-left: auto;">${quando}</span>
        </div>
        <p style="margin-top: var(--space-2);">Operação: <span class="num">${esc(ev.operation)}</span></p>
        <p class="muted">${esc(ev.message)}</p>
        ${ev.details ? `<p class="muted num">${esc(ev.details)}</p>` : ""}
      `;
      lista.appendChild(li);
    });
  } catch (e) {
    console.error("Falha ao ler eventos:", e);
    lista.innerHTML = "";
    lista.appendChild(mensagemVazia("Não foi possível ler o histórico"));
  }
}

function rotularCategoria(categoria: AppEvent["category"]): string {
  const nomes: Record<AppEvent["category"], string> = {
    sistema: "Sistema",
    deteccao: "Detecção",
    configuracao: "Configuração",
    otimizacao: "Otimização",
    diagnostico: "Diagnóstico",
  };
  return nomes[categoria] ?? categoria;
}

function esc(value: string | null | undefined): string {
  if (value == null) return INDISPONIVEL;
  const div = document.createElement("div");
  div.textContent = value;
  return div.innerHTML;
}

function mensagemVazia(message: string): HTMLElement {
  const p = document.createElement("p");
  p.className = "muted";
  p.textContent = message;
  return p;
}

export type { SystemSnapshot };
