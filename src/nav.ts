const TAB_TITLES: Record<string, string> = {
  "visao-geral": "Apps",
  desempenho: "Hardware e Sistema",
  kernels: "Kernels e Drivers",
  otimizacao: "Otimização",
  historico: "Histórico e Diagnósticos",
};

const TAB_IDS = Object.keys(TAB_TITLES);

const scrollPagina: Record<string, number> = {};

/** troca a página visível guardando/restaurando a rolagem de cada uma */
export function mostrarPagina(qual: string): void {
  document.querySelectorAll<HTMLDivElement>(".page").forEach((p) => {
    const dentro = p.classList.contains("page--active");
    if (dentro) scrollPagina[p.id] = p.scrollTop;
    const vaiFicar = p.id === `page-${qual}`;
    if (vaiFicar && !dentro) {
      p.classList.add("page--active");
      p.scrollTop = scrollPagina[p.id] ?? 0;
    } else if (!vaiFicar && dentro) {
      p.classList.remove("page--active");
    }
  });
  const tab = TAB_IDS.includes(qual) ? qual : qual.startsWith("app-") ? "visao-geral" : qual;
  document.querySelectorAll<HTMLButtonElement>("button[data-tab]").forEach((b) => {
    b.classList.toggle("nav__item--active", b.dataset.tab === tab);
  });
  const topbarTitle = document.getElementById("topbar-title");
  if (topbarTitle) topbarTitle.textContent = TAB_TITLES[tab] ?? topbarTitle.textContent;
}

export function initNav(onChange?: (tab: string) => void): void {
  const buttons = document.querySelectorAll<HTMLButtonElement>("button[data-tab]");

  buttons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const tab = btn.dataset.tab;
      if (!tab) return;
      mostrarPagina(tab);
      onChange?.(tab);
    });
  });

  // arranque: tudo no topo
  document.querySelectorAll<HTMLDivElement>(".page").forEach((p) => {
    p.scrollTop = 0;
  });
}
