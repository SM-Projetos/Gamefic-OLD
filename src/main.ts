import { applyTheme, applyReducedMotion } from "./motion";
import { initNav } from "./nav";
import { loadPrefs, savePrefs } from "./prefs";
import { getSystemInfo, renderKernels, renderVisaoGeral } from "./system";
import type { SystemSnapshot } from "./system";
import type { ThemeChoice } from "./motion";

window.addEventListener("DOMContentLoaded", async () => {
  const statusIndicator = document.getElementById("status-indicator");

  const setStatus = (kind: string, text: string) => {
    if (statusIndicator) {
      statusIndicator.innerHTML = `<span class="pill pill--${kind}" id="pill-warning-${kind}"></span>`;
      const pill = statusIndicator.firstElementChild as HTMLElement | null;
      if (pill) pill.textContent = text;
    }
  };

  setStatus("info", "Detectando sistema...");

  getSystemInfo()
    .then((snapshot: SystemSnapshot) => {
      renderVisaoGeral(snapshot);
      renderKernels(snapshot);
      setStatus("ok", "Sistema detectado");
    })
    .catch((e) => {
      console.error("Falha na detecção do sistema:", e);
      setStatus("error", "Detecção indisponível");
    });

  const prefs = await loadPrefs();
  applyTheme(prefs.theme);
  applyReducedMotion(prefs.reducedMotion);

  initNav();

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
