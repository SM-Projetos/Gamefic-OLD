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
  name: string;
  description: string;
  available: boolean;
}

export interface SystemSnapshot {
  distro?: DistroInfo | null;
  kernel?: string | null;
  arch?: string | null;
  cpu?: CpuInfo | null;
  memory?: MemoryInfo | null;
  gpus: GpuInfo[];
  vulkan?: VulkanInfo | null;
  temperatures: SensorReading[];
  tools: ToolInfo[];
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

function listItems(items: string[]): string {
  return items.join(" · ");
}

export function renderVisaoGeral(snapshot: SystemSnapshot): void {
  setText("#hw-cpu-modelo", snapshot.cpu?.model ?? null);
  setText(
    "#hw-cpu-detalhe",
    summarizarCpu(snapshot.cpu),
  );
  setText("#hw-gpu", summarizarGpus(snapshot.gpus));
  setText("#hw-memoria", summarizarMemoria(snapshot.memory));
  setText("#hw-temperatura", summarizarTemperaturas(snapshot.temperatures));
  setText(
    "#hw-kernel",
    summarizarKernel(snapshot.distro, snapshot.kernel, snapshot.arch),
  );
  setText("#hw-ferramentas", summarizarFerramentasHabilitadas(snapshot.tools));
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

function summarizarCpu(cpu?: CpuInfo | null): string {
  if (!cpu) return INDISPONIVEL;
  const partes: string[] = [];
  if (cpu.coresPhysical != null && cpu.threadsLogical != null) {
    partes.push(`${cpu.coresPhysical} núcleos / ${cpu.threadsLogical} threads`);
  } else if (cpu.threadsLogical != null) {
    partes.push(`${cpu.threadsLogical} threads`);
  }
  if (cpu.usagePercent != null) {
    partes.push(`uso atual ${(cpu.usagePercent).toFixed(0)}%`);
  }
  if (cpu.currentFreqsMhz.length > 0) {
    partes.push(`frequências ${mhz(cpu.currentFreqsMhz[0])}–${mhz(cpu.currentFreqsMhz[cpu.currentFreqsMhz.length - 1])}`);
  }
  return partes.join(" · ") ?? INDISPONIVEL;
}

function summarizarGpus(gpus: GpuInfo[]): string {
  if (!gpus || gpus.length === 0) return INDISPONIVEL;
  return listItems(
    gpus.map((g) => `${g.deviceName ?? g.vendorName ?? INDISPONIVEL}${g.driver ? ` (driver ${g.driver})` : ""}`),
  );
}

function summarizarMemoria(mem?: MemoryInfo | null): string {
  if (!mem) return INDISPONIVEL;
  const total = gib(mem.totalKb);
  const used = gib(mem.usedKb);
  const partes = [];
  if (mem.usedKb != null && mem.totalKb != null) {
    partes.push(`${used} de ${total} (${((mem.usedKb / mem.totalKb) * 100).toFixed(0)}%)`);
  } else if (mem.totalKb != null) {
    partes.push(`total ${total}`);
  }

  if (mem.swapTotalKb != null && mem.swapUsedKb != null && mem.swapTotalKb > 0) {
    partes.push(`swap ${gib(mem.swapUsedKb)} de ${gib(mem.swapTotalKb)}`);
  }
  return partes.length > 0 ? listItems(partes) : total;
}

function summarizarTemperaturas(temps: SensorReading[]): string {
  if (!temps || temps.length === 0) return INDISPONIVEL;
  return listItems(temps.slice(0, 6).map((t) => `${t.chip}/${t.label}: ${temperatura(t.celsius)}`));
}

function summarizarKernel(
  distro?: DistroInfo | null,
  kernel?: string | null,
  arch?: string | null,
): string {
  if (!distro && !kernel && !arch) return INDISPONIVEL;
  const partes = [distro?.name, kernel ? `kernel ${kernel}` : null, arch].filter(Boolean);
  return partes.length > 0 ? listItems(partes as string[]) : INDISPONIVEL;
}

function summarizarFerramentasHabilitadas(tools: ToolInfo[]): string {
  if (!tools || tools.length === 0) return INDISPONIVEL;
  const installed = tools.filter((t) => t.available);
  if (installed.length === 0) return "Nenhuma identificada";
  return listItems(installed.map((t) => t.name));
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
