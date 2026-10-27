import { invoke } from "@tauri-apps/api/core";

export interface DistroInfo {
  name: string;
  version?: string | null;
  family?: string | null;
}

export interface CpuInfo {
  model?: string | null;
  vendor?: string | null;
  coresPhysical?: number | null;
  threadsLogical?: number | null;
  baseFreqMhz?: number | null;
  currentFreqsMhz: number[];
  usagePercent?: number | null;
}

export interface MemoryInfo {
  totalKb?: number | null;
  usedKb?: number | null;
  availableKb?: number | null;
  swapTotalKb?: number | null;
  swapUsedKb?: number | null;
}

export interface GpuInfo {
  pciAddress?: string | null;
  vendorId?: string | null;
  vendorName?: string | null;
  deviceId?: string | null;
  deviceName?: string | null;
  driver?: string | null;
  driverVersion?: string | null;
}

export interface VulkanGpu {
  deviceName?: string | null;
  driverName?: string | null;
}

export interface VulkanInfo {
  gpus: VulkanGpu[];
}

export interface SensorReading {
  chip: string;
  label: string;
  celsius: number;
}

export interface ToolInfo {
  id: string;
  name: string;
  description: string;
  available: boolean;
  version?: string | null;
  icon?: string | null;
}

export interface StorageDevice {
  name: string;
  model?: string | null;
  kind?: string | null;
  sizeBytes?: number | null;
  usedBytes?: number | null;
  totalBytes?: number | null;
}

export interface AppEntry {
  id?: string;
  name: string;
  description: string;
  available: boolean;
  version?: string | null;
  icon?: string | null;
  estado?: string | null; // "instalado" | "nao_instalado" | "desconhecido"
  formato?: string | null;
}

export interface SystemSnapshot {
  distro?: DistroInfo | null;
  uptimeSeconds?: number | null;
  storage: StorageDevice[];
  kernel?: string | null;
  arch?: string | null;
  cpu?: CpuInfo | null;
  cpuTempC?: number | null;
  mesaVersion?: string | null;
  memory?: MemoryInfo | null;
  gpus: GpuInfo[];
  vulkan?: VulkanInfo | null;
  temperatures: SensorReading[];
  tools: ToolInfo[];
  launchers: AppEntry[];
  emuladores: AppEntry[];
  compatibilidade: AppEntry[];
}

export const INDISPONIVEL = "Não disponível";

export function getSystemInfo(): Promise<SystemSnapshot> {
  return invoke<SystemSnapshot>("get_system_info");
}

export function mhz(mhz: number | null | undefined): string {
  if (mhz == null || mhz <= 0) return INDISPONIVEL;
  return mhz >= 1000 ? `${(mhz / 1000).toFixed(1)} GHz` : `${mhz} MHz`;
}

export function gib(kb: number | null | undefined): string {
  if (kb == null || kb <= 0) return INDISPONIVEL;
  return `${(kb / (1024 * 1024)).toFixed(1)} GiB`;
}

export function temperatura(celsius: number): string {
  return `${celsius.toFixed(1)} °C`;
}

export function renderVisaoGeral(snapshot: SystemSnapshot): void {
  renderTiles(snapshot.launchers, "ov-launchers", "Nenhum launcher instalado");
  renderTiles(snapshot.tools, "ov-tools", "Nenhuma ferramenta de jogos detectada");
  renderTiles(snapshot.compatibilidade, "ov-compat", "Nenhuma ferramenta de compatibilidade detectada");
  renderTiles(snapshot.emuladores, "ov-emul", "Nenhum emulador detectado");
  inserirCardAddCompat();
  renderDesempenho(snapshot);
}

function inserirCardAddCompat(): void {
  const grid = document.getElementById("ov-compat");
  if (!grid) return;
  const add = document.createElement("div");
  add.className = "tool-tile tool-tile--add";
  add.setAttribute("aria-label", "Adicionar compatibilidade");
  add.innerHTML = `
    <span aria-hidden="true">+</span>
    <span class="tool-tile__name" style="min-height:0">Adicionar</span>
  `;
  grid.appendChild(add);
}

export function renderDesempenho(snapshot: SystemSnapshot): void {
  // CPU
  setText("#dp-cpu-modelo", snapshot.cpu?.model ?? null);
  setText("#dp-cpu-detalhe", resumoCpu(snapshot.cpu));
  setText("#dp-cpu-frequencia", snapshot.cpu?.currentFreqsMhz?.length ? mhz(snapshot.cpu.currentFreqsMhz[0]) : null);
  setTextTemp("#dp-cpu-temperatura", snapshot.cpuTempC);

  // GPU (driver + versao, agrupados por GPU)
  const gpu = snapshot.gpus[0] ?? null;
  setText("#dp-gpu-modelo", gpu?.deviceName ?? gpu?.vendorName ?? (snapshot.gpus.length ? null : "Nenhuma GPU identificada"));

  // Memoria e armazenamento
  renderMemoria(snapshot.memory);
  renderStorage(snapshot.storage);

  // Sistema
  setText("#dp-distro", snapshot.distro?.name ?? null);
  setText("#dp-kernel", snapshot.kernel ?? null);
  setText("#dp-arch", snapshot.arch ?? null);
  const mesa = snapshot.mesaVersion?.trim();
  setText("#dp-mesa", mesa ? `Mesa ${mesa.replace(/^Mesa\s*/i, "")}` : null);
  const uptime = document.getElementById("dp-uptime");
  if (uptime) {
    uptime.textContent =
      snapshot.uptimeSeconds && snapshot.uptimeSeconds > 0
        ? formatarUptime(snapshot.uptimeSeconds)
        : INDISPONIVEL;
  }
  renderDrivers(snapshot.gpus);
}

function setTextTemp(selector: string, celsius: number | null | undefined): void {
  setText(selector, celsius != null ? temperatura(celsius) : null);
}

function renderStorage(storage: StorageDevice[]): void {
  const container = document.getElementById("dp-storage");
  if (!container) return;
  container.innerHTML = "";
  if (!storage || storage.length === 0) {
    const vazio = document.createElement("p");
    vazio.className = "muted";
    vazio.textContent = `${INDISPONIVEL} — nenhuma unidade de armazenamento detectada`;
    container.appendChild(vazio);
    return;
  }
  storage.forEach((d) => {
    const unidade = document.createElement("div");
    unidade.className = "storage-unit";
    const usada = d.usedBytes ?? null;
    const total = d.totalBytes ?? d.sizeBytes ?? null;

    const head = document.createElement("div");
    head.className = "storage-unit__head";
    const nameBox = document.createElement("span");
    nameBox.className = "storage-unit__media storage-list__media";
    nameBox.innerHTML = `
      <span class="storage-unit__name num">${esc(d.name)}</span>
      <span class="storage-unit__model">${esc(d.model ?? "")}</span>
    `;
    const kind = document.createElement("span");
    kind.className = "storage-unit__kind";
    kind.textContent = d.kind ?? "";
    head.append(nameBox, kind);

    if (usada != null && total != null && total > 0) {
      const bar = document.createElement("div");
      bar.className = "bar";
      const fill = document.createElement("div");
      fill.className = "bar__fill";
      const pct = Math.max(0, Math.min(100, (usada / total) * 100));
      fill.style.width = `${pct.toFixed(1)}%`;
      bar.appendChild(fill);
      unidade.appendChild(bar);
    }

    const foot = document.createElement("div");
    foot.className = "storage-unit__foot";
    foot.innerHTML =
      usada != null
        ? `<span>Usada</span><span class="num">${esc(formatarBytes(usada) ?? INDISPONIVEL)}</span>`
        : `<span>Partições não montadas — espaço usado indisponível</span>`;
    if (total != null) {
      const tot = document.createElement("span");
      tot.innerHTML = `<span class="num">${esc(formatarBytes(total) ?? "")}</span>`;
      foot.appendChild(tot);
    }

    unidade.append(head, foot);
    container.appendChild(unidade);
  });
}

function renderDrivers(gpus: GpuInfo[]): void {
  const dl = document.getElementById("dp-drivers");
  if (!dl) return;
  dl.innerHTML = "";
  if (!gpus || gpus.length === 0) {
    dl.innerHTML = `<dt>Nenhuma GPU identificada</dt><dd class="muted">—</dd>`;
    return;
  }
  gpus.forEach((g) => {
    const dt = document.createElement("dt");
    dt.textContent = g.deviceName ?? g.vendorName ?? INDISPONIVEL;
    const dd = document.createElement("dd");
    dd.className = "num";
    const driver = g.driver ?? INDISPONIVEL;
    if (g.driverVersion) {
      const labelMesa = g.driver !== "nvidia" ? "Mesa " : "";
      const limpo = g.driverVersion.replace(/^Mesa\s*/, "");
      dd.textContent = `${driver} · ${labelMesa}${limpo}`;
    } else {
      dd.textContent = driver;
    }
    dl.append(dt, dd);
  });
}

function formatarBytes(bytes: number | null | undefined): string | null {
  if (bytes == null || bytes <= 0) return null;
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value >= 100 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function renderMemoria(mem?: MemoryInfo | null): void {
  const total = document.getElementById("dp-ram-total");
  if (total) {
    total.textContent =
      mem?.usedKb != null && mem?.totalKb != null
        ? `${gib(mem.usedKb)} / ${gib(mem.totalKb)}`
        : INDISPONIVEL;
  }

  const pct =
    mem?.usedKb != null && mem?.totalKb != null && mem.totalKb > 0
      ? (mem.usedKb / mem.totalKb) * 100
      : null;

  const fill = document.querySelector<HTMLElement>("#dp-ram-bar .bar__fill");
  if (fill && pct != null) fill.style.width = `${Math.max(0, Math.min(100, pct)).toFixed(1)}%`;

  const pctLabel = document.getElementById("dp-ram-pct");
  if (pctLabel) pctLabel.textContent = pct != null ? `${pct.toFixed(0)}% em uso` : INDISPONIVEL;

  setText("#dp-ram-disponivel", mem?.availableKb != null ? gib(mem.availableKb) : null);

  const swap =
    mem?.swapTotalKb != null && mem?.swapUsedKb != null && mem.swapTotalKb > 0
      ? `${gib(mem.swapUsedKb)} de ${gib(mem.swapTotalKb)}`
      : null;
  setText("#dp-swap-uso", swap ?? (mem?.swapTotalKb === 0 ? "Sem swap ativo" : null));
}

function resumoCpu(cpu?: CpuInfo | null): string | null {
  if (!cpu) return null;
  const partes: string[] = [];
  if (cpu.coresPhysical != null && cpu.threadsLogical != null) {
    partes.push(`${cpu.coresPhysical} núcleos / ${cpu.threadsLogical} threads`);
  } else if (cpu.threadsLogical != null) {
    partes.push(`${cpu.threadsLogical} threads`);
  }
  if (cpu.baseFreqMhz != null) partes.push(`base ${mhz(cpu.baseFreqMhz)}`);
  return partes.length > 0 ? partes.join(" · ") : null;
}

export function atualizarRing(
  circleId: string,
  numId: string,
  percento: number | null,
): void {
  const circle = document.getElementById(circleId) as SVGCircleElement | null;
  const num = document.getElementById(numId);
  if (num && percento != null) num.textContent = `${percento.toFixed(0)}%`;
  if (!circle) return;
  const C = 2 * Math.PI * 36;
  const pct = percento == null ? 0 : Math.max(0, Math.min(100, percento)) / 100;
  circle.style.strokeDasharray = `${C}`;
  circle.style.strokeDashoffset = `${C * (1 - pct)}`;
}

export function classificarTemperatura(c: number): { classe: string; texto: string } {
  if (c >= 75) return { classe: "critical", texto: "Crítica" };
  if (c >= 60) return { classe: "hot", texto: "Atenção" };
  return { classe: "ok", texto: "Normal" };
}

function renderTiles(entries: AppEntry[], gridId: string, emptyMsg: string): void {
  const grid = document.getElementById(gridId);
  if (!grid) return;
  grid.innerHTML = "";
  if (!entries || entries.length === 0) {
    grid.innerHTML = `<p class="muted tools-grid__empty">${esc(emptyMsg)}</p>`;
    return;
  }
  entries.forEach((entry, i) => {
    const estado = entry.estado ?? (entry.available ? "instalado" : "nao_instalado");
    const rotuloEstado =
      estado === "instalado"
        ? `✔ Instalado${entry.formato ? ` · ${entry.formato}` : ""}`
        : estado === "desconhecido"
          ? "Verificação indisponível"
          : "Não instalado";
    const tile = document.createElement("div");
    tile.className = `tool-tile${estado === "instalado" ? "" : " tool-tile--off"}`;
    tile.dataset.appId = entry.id ?? "";
    const iconHtml = entry.icon
      ? `<span class="tool-tile__icon" aria-hidden="true"><img class="tool-tile__img" src="${esc(entry.icon)}" alt=""></span>`
      : `<span class="tool-tile__icon tool-tile__icon--purple" aria-hidden="true">${esc(inicial(entry.name, i))}</span>`;
    const versionHtml =
      estado === "instalado" && entry.version
        ? ` <span class="tool-tile__version num">${esc(entry.version)}</span>`
        : "";
    tile.innerHTML = `
      ${iconHtml}
      <span class="tool-tile__name">${esc(entry.name)}${versionHtml}</span>
      <span class="tool-tile__status ${estado === "instalado" ? "" : "tool-tile__status--off"}">
        ${esc(rotuloEstado)}
      </span>
    `;
    grid.appendChild(tile);
  });
}

function inicial(nome: string, indice: number): string {
  const chave = nome.trim();
  if (!chave) return "?";
  if (indice % 2 === 0) return chave.charAt(0).toUpperCase();
  return chave.charAt(0).toLowerCase();
}

export function formatarUptime(segundos: number): string {
  const h = Math.floor(segundos / 3600);
  const m = Math.floor((segundos % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return `${segundos}s`;
}

export function renderKernels(snapshot: SystemSnapshot): void {
  setText("#kd-distro", snapshot.distro?.name ?? null);
  setText("#kd-familia", snapshot.distro?.family ?? null);
  setText("#kd-versao-distribuicao", snapshot.distro?.version ?? null);
  setText("#kd-kernel", snapshot.kernel ?? null);
  setText("#kd-arch", snapshot.arch ?? null);

  if (snapshot.cpu) {
    setText("#kd-cpu-modelo", snapshot.cpu.model ?? null);
    setText("#kd-cpu-fisicos", contagem(snapshot.cpu.coresPhysical));
    setText("#kd-cpu-threads", contagem(snapshot.cpu.threadsLogical));
    setText("#kd-cpu-frequencia", mhz(snapshot.cpu.baseFreqMhz));
  }

  const container = document.getElementById("kd-gpu-lista");
  if (container) {
    container.innerHTML = "";
    const gpus = snapshot.gpus;
    if (gpus.length === 0) {
      container.appendChild(empty("Nenhuma GPU identificada"));
    } else {
      gpus.forEach((gpu) => {
        container.appendChild(cardGpu(gpu));
      });
    }
  }

  const vulkanContainer = document.getElementById("kd-vulkan-lista");
  if (vulkanContainer) {
    vulkanContainer.innerHTML = "";
    const vulkan = snapshot.vulkan;
    if (!vulkan || vulkan.gpus.length === 0) {
      vulkanContainer.appendChild(
        alerta("vulkaninfo não disponível no sistema — suporte Vulkan não pôde ser consultado"),
      );
    } else {
      vulkan.gpus.forEach((v) => {
        const item = document.createElement("li");
        item.innerHTML = `
          <span class="kv"><dt>Dispositivo</dt><dd>${esc(v.deviceName)}</dd></span>
          <span class="kv"><dt>Driver</dt><dd>${esc(v.driverName ?? INDISPONIVEL)}</dd></span>
        `;
        vulkanContainer.appendChild(item);
      });
    }
  }

  const ferramentasContainer = document.getElementById("kd-ferramentas-lista");
  if (ferramentasContainer) {
    ferramentasContainer.innerHTML = "";
    (snapshot.tools ?? []).forEach((tool) => {
      const li = document.createElement("li");
      li.className = "kv-lista-item";
      li.innerHTML = `
        <span class="kv"><dt>${esc(tool.name)}</dt><dd><span class="pill ${
          tool.available ? "pill--ok" : "pill--info"
        }">${tool.available ? "Instalada" : "Não instalada"}</span><span style="margin-left: 8px; color: var(--text-dim); font-size: var(--text-aux);">${esc(tool.description)}</span></dd></span>
      `;
      ferramentasContainer.appendChild(li);
    });
  }
}

function contagem(n?: number | null): string {
  return n == null || n <= 0 ? INDISPONIVEL : String(n);
}

function empty(message: string): HTMLElement {
  const item = document.createElement("p");
  item.className = "muted";
  item.textContent = message;
  return item;
}

function alerta(message: string): HTMLElement {
  const item = document.createElement("div");
  item.className = "alert alert--warn";
  item.textContent = message;
  return item;
}

function cardGpu(gpu: GpuInfo): HTMLElement {
  const card = document.createElement("div");
  card.className = "card";
  card.innerHTML = `
    <div class="card-head"><span class="pill pill--info">${esc(gpu.vendorName ?? INDISPONIVEL)}</span></div>
    <dl class="kv">
      <dt>Modelo</dt><dd>${esc(gpu.deviceName ?? INDISPONIVEL)}</dd>
      <dt>Endereço PCI</dt><dd class="num">${esc(gpu.pciAddress ?? INDISPONIVEL)}</dd>
      <dt>IDs</dt><dd class="num">${esc(gpu.vendorId ?? "?")}:${esc(gpu.deviceId ?? "?")}</dd>
      <dt>Driver</dt><dd class="num">${esc(gpu.driver ?? INDISPONIVEL)}</dd>
    </dl>
  `;
  return card;
}

function setText(selector: string, value: string | null | undefined): void {
  const element = document.querySelector(selector);
  if (!element) return;
  element.textContent = value ?? INDISPONIVEL;
}

function esc(value: string | null | undefined): string {
  if (value == null) return INDISPONIVEL;
  const div = document.createElement("div");
  div.textContent = value;
  return div.innerHTML;
}
