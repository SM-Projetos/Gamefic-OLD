const TAB_TITLES: Record<string, string> = {
  "visao-geral": "Visão geral",
  desempenho: "Hardware e Sistema",
  kernels: "Kernels e Drivers",
  otimizacao: "Otimização",
  historico: "Histórico e Diagnósticos",
};

const TAB_IDS = Object.keys(TAB_TITLES);

export function initNav(onChange?: (tab: string) => void): void {
  const buttons = document.querySelectorAll<HTMLButtonElement>("button[data-tab]");
  const topbarTitle = document.getElementById("topbar-title");

  buttons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const tab = btn.dataset.tab;
      if (!tab) return;
      activateTab(tab);
      onChange?.(tab);
    });
  });

  function activateTab(tab: string) {
    TAB_IDS.forEach((t) => {
      document.getElementById(`page-${t}`)?.classList.toggle("page--active", t === tab);
    });
    buttons.forEach((b) => {
      b.classList.toggle("nav__item--active", b.dataset.tab === tab);
    });
    if (topbarTitle) topbarTitle.textContent = TAB_TITLES[tab] ?? tab;
  }
}
