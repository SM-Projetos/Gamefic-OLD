import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { applyTheme, applyReducedMotion } from "./motion";
import { initNav } from "./nav";
import { loadPrefs, savePrefs } from "./prefs";
import { wireAppClicks } from "./apps";
import { getSystemInfo, renderKernels, renderVisaoGeral } from "./system";
import { Monitor } from "./monitor";
import {
  clearEvents,
  getAnalysis,
  renderHistorico,
  renderOtimizacao,
  recordEvent,
} from "./historico";
import type { SystemSnapshot } from "./system";
import type { ThemeChoice } from "./motion";

window.addEventListener("DOMContentLoaded", async () => {
  getVersion().then((v) => {
    const el = document.getElementById("app-version");
    if (el) el.textContent = `v${v}`;
  }).catch(() => {});

  const monitorEstado = document.getElementById("monitor-estado");

  const setMonitorEstado = (text: string) => {
    if (monitorEstado) monitorEstado.textContent = text;
  };
  const statusIndicator = document.getElementById("status-indicator");

  const setStatus = (kind: string, text: string) => {
    if (statusIndicator) {
      statusIndicator.innerHTML = `<span class="pill pill--${kind}"></span>`;
      const pill = statusIndicator.firstElementChild as HTMLElement | null;
      if (pill) pill.textContent = text;
    }
  };

  const prefs = await loadPrefs();
  applyTheme(prefs.theme);
  applyReducedMotion(prefs.reducedMotion);
  document.body.classList.remove("hidden-inicial");

  setStatus("info", "Detectando sistema...");

  wireAppClicks();

  let ultimoSnapshot: SystemSnapshot | null = null;

  const recarregarSistema = () => {
    if (!ultimoSnapshot) return;
    getSystemInfo()
      .then((snapshot: SystemSnapshot) => {
        ultimoSnapshot = snapshot;
        renderVisaoGeral(snapshot);
        renderKernels(snapshot);
      })
      .catch(() => {});
  };

  const verificarIcones = () => {
    invoke<number>("refresh_tool_icons")
      .then((atualizados: number) => {
        recordEvent(
          "manutencao",
          "ícones de ferramentas",
          "concluido",
          `Verificação de ícones concluída (${atualizados} atualizado(s))`,
        );
        if (atualizados > 0) recarregarSistema();
      })
      .catch(() => {});
  };

  getSystemInfo()
    .then((snapshot: SystemSnapshot) => {
      ultimoSnapshot = snapshot;
      renderVisaoGeral(snapshot);
      renderKernels(snapshot);
      setStatus("ok", "Sistema detectado");
      recordEvent("deteccao", "detecção do sistema", "concluido", "Hardware e sistema identificados com as fontes nativas");
      // Verifica ícones mais recentes em segundo plano e re-renderiza se mudar algo
      window.setTimeout(verificarIcones, 15_000);
      window.setInterval(verificarIcones, 6 * 60 * 60 * 1000);
    })
    .catch((e) => {
      console.error("Falha na detecção do sistema:", e);
      setStatus("error", "Detecção indisponível");
      recordEvent("deteccao", "detecção do sistema", "falha", `Detecção não concluída: ${String(e)}`);
    });

  getAnalysis()
    .then(renderOtimizacao)
    .catch((e) => console.error("Falha na análise de otimização:", e));

  // estados de instalação mudaram (instale/remova/cancele) → recarrega os cards
  window.addEventListener("estado-apps-mudou", () => recarregarSistema());

  renderHistorico().catch((e) => console.error(e));
  wireHistorico();

  initNav((tab) => {
    if (tab === "desempenho") monitor.render();
  });

  let filaReinicioMonitoramento: Promise<void> = Promise.resolve();
  const reiniciarMonitoramento = (intervalMs: number): Promise<boolean> => {
    const operacao = filaReinicioMonitoramento.then(async () => {
      await invoke("stop_monitoring");
      await invoke("start_monitoring", { intervalMs });
    });
    filaReinicioMonitoramento = operacao.catch((e) => {
      console.error("Falha ao reiniciar monitoramento:", e);
    });
    return operacao.then(() => true, () => false);
  };

  const monitor = new Monitor((intervalMs) => {
    prefs.monitorIntervalMs = intervalMs;
    savePrefs(prefs);
    void reiniciarMonitoramento(intervalMs).then((iniciou) => {
      setMonitorEstado(iniciou ? "Coleta ativa" : "Monitoramento indisponível");
    });
  });

  const selectInterval = document.getElementById(
    "select-intervalo",
  ) as HTMLSelectElement | null;
  if (selectInterval) {
    selectInterval.value = String(prefs.monitorIntervalMs);
    selectInterval.addEventListener("change", () => {
      monitor.setInterval(Number(selectInterval.value));
    });
  }

  monitor
    .init()
    .then(() => {
      monitor.show(document.getElementById("page-desempenho"));
      void reiniciarMonitoramento(prefs.monitorIntervalMs).then((iniciou) => {
        setMonitorEstado(iniciou ? "Coleta ativa" : "Monitoramento indisponível");
      });
    })
    .catch((e) => {
      console.error("Falha na coleta de métricas:", e);
      setMonitorEstado("Monitoramento indisponível");
    });

  const settingsPanel = document.getElementById("settings-panel");
  const btnSettings = document.getElementById("btn-settings");
  const btnClose = document.getElementById("btn-settings-close");
  const selectTheme = document.getElementById("select-tema") as HTMLSelectElement | null;
  const switchMotion = document.getElementById("switch-movimento") as HTMLInputElement | null;

  if (selectTheme) {
    selectTheme.value = prefs.theme;
    selectTheme.addEventListener("change", () => {
      const choice = (selectTheme.value as ThemeChoice) ?? "dark";
      applyTheme(choice);
      prefs.theme = choice;
      savePrefs(prefs);
    });
  }

  if (switchMotion) {
    switchMotion.checked = prefs.reducedMotion;
    switchMotion.addEventListener("change", () => {
      const enabled = switchMotion.checked;
      applyReducedMotion(enabled);
      prefs.reducedMotion = enabled;
      savePrefs(prefs);
    });
  }

  if (settingsPanel && btnSettings && btnClose) {
    const open = () => {
      settingsPanel.classList.add("settings-panel--open");
      btnClose.focus();
    };
    const close = () => {
      settingsPanel.classList.remove("settings-panel--open");
      btnSettings.focus();
    };
    btnSettings.addEventListener("click", open);
    btnClose.addEventListener("click", close);
    settingsPanel.addEventListener("click", (e) => {
      if (e.target === settingsPanel) close();
    });
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && settingsPanel.classList.contains("settings-panel--open")) close();
    });
  }
});

function wireHistorico(): void {
  const filtroCategoria = document.getElementById("filtro-categoria");
  const filtroResultado = document.getElementById("filtro-resultado");
  const btnLimpar = document.getElementById("btn-limpar-historico");

  [filtroCategoria, filtroResultado].forEach((el) =>
    el?.addEventListener("change", () => {
      renderHistorico().catch((e) => console.error(e));
    }),
  );

  btnLimpar?.addEventListener("click", () => {
    const confirmed = window.confirm(
      "Isso removerá permanentemente todos os eventos registrados do histórico local. Confirma?",
    );
    if (!confirmed) return;

    clearEvents()
      .then(() => {
        recordEvent(
          "sistema",
          "limpar histórico",
          "concluido",
          "Histórico local de eventos removido a pedido do usuário",
        );
        renderHistorico().catch((e) => console.error(e));
      })
      .catch((e) => console.error("Falha ao limpar histórico:", e));
  });
}
