const TAB_TITLES: Record<string, string> = {
  "visao-geral": "Apps",
  desempenho: "Hardware e Sistema",
  kernels: "Kernels e Drivers",
  otimizacao: "Otimização",
  historico: "Histórico e Diagnósticos",
};

const TAB_IDS = Object.keys(TAB_TITLES);

const scrollPagina: Record<string, number> = {};

/** troca a página visível guardando/restaurando a rolagem de cada uma.
 * A rolagem vive no .content (contêiner único); guardar/definir lá preserva a posição por página */
export function mostrarPagina(qual: string): void {
  const content = document.querySelector<HTMLDivElement>(".content");
  const pages = Array.from(document.querySelectorAll<HTMLDivElement>(".page"));
  // 1ª passada: guarda a rolagem atual na página que está saindo
  pages.forEach((p) => {
    if (p.classList.contains("page--active") && p.id !== `page-${qual}`) {
      scrollPagina[p.id] = content?.scrollTop ?? 0;
    }
  });
  // 2ª passada: troca as páginas e restaura a rolagem da que entra
  pages.forEach((p) => {
    const dentro = p.classList.contains("page--active");
    const vaiFicar = p.id === `page-${qual}`;
    if (vaiFicar && !dentro) {
      p.classList.add("page--active");
      if (content) content.scrollTop = scrollPagina[p.id] ?? 0;
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
  document.querySelector<HTMLDivElement>(".content")?.scrollTo({ top: 0 });
}
