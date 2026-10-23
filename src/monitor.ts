import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import { drawChart, RollingData } from "./charts";
import { atualizarRing, gib } from "./system";

export interface MetricSample {
  timestampMs: number;
  cpuPercent: number | null;
  cpuFreqAvgMhz: number | null;
  memUsedPercent: number | null;
  memUsedKb: number | null;
  memTotalKb: number | null;
  swapUsedPercent: number | null;
  tempMaxC: number | null;
  tempLabel: string | null;
  gpuUtilizationPercent: number | null;
  gpuMemUsedMb: number | null;
  gpuMemTotalMb: number | null;
  gpuTempC: number | null;
}

const HISTORY = 300;

const CPU_STYLE = { stroke: "#a78bfa", fill: "rgba(167, 139, 250, 0.14)" };
const GPU_STYLE = { stroke: "#22d3ee", fill: "rgba(34, 211, 238, 0.12)" };

interface ChartCard {
  canvasId: string;
  data: RollingData;
  style: { stroke: string; fill: string };
  unit: string;
}

export class Monitor {
  private charts: Record<string, ChartCard>;
  private unlisten: UnlistenFn | null = null;
  private intervalMs = 2000;
  private resizeObserver: ResizeObserver | null = null;

  constructor(
    private readonly onChangeInterval: (intervalMs: number) => void,
  ) {
    this.charts = {
      cpu: { canvasId: "dp-cpu-spark", data: new RollingData(HISTORY), style: CPU_STYLE, unit: "%" },
      gpu: { canvasId: "dp-gpu-spark", data: new RollingData(HISTORY), style: GPU_STYLE, unit: "%" },
      vram: { canvasId: "dp-gpu-vram", data: new RollingData(HISTORY), style: GPU_STYLE, unit: "%" },
    };
  }

  private canvas(id: string): HTMLCanvasElement | null {
    const el = document.getElementById(id);
    return el instanceof HTMLCanvasElement ? el : null;
  }

  async init(): Promise<void> {
    const samples = await invoke<MetricSample[]>("get_history");
    samples.forEach((s) => this.pushSample(s));

    this.unlisten = await listen<MetricSample>("metrics", (event) => {
      this.pushSample(event.payload);
    });
  }

  getInterval(): number {
    return this.intervalMs;
  }

  setInterval(intervalMs: number): void {
    this.intervalMs = intervalMs;
    this.onChangeInterval(intervalMs);
  }

  hide(): void {
    this.resizeObserver?.disconnect();
  }

  show(canvasHost: HTMLElement | null = null): void {
    if (canvasHost) {
      if (!this.resizeObserver) {
        this.resizeObserver = new ResizeObserver(() => this.render());
      }
      this.resizeObserver.observe(canvasHost);
    }
    this.render();
  }

  dispose(): void {
    this.unlisten?.();
    this.unlisten = null;
    this.resizeObserver?.disconnect();
  }

  render(): void {
    Object.values(this.charts).forEach((c) => this.renderChart(c));
  }

  private renderChart(card: ChartCard): void {
    const canvas = this.canvas(card.canvasId);
    if (!canvas || !canvas.isConnected) return;
    drawChart(canvas, card.data.values(), 100, card.style, "%", true);
  }

  private pushSample(sample: MetricSample): void {
    this.charts.cpu.data.push(sample.cpuPercent);
    this.charts.gpu.data.push(sample.gpuUtilizationPercent);
    const vramPct =
      sample.gpuMemUsedMb == null
        ? null
        : sample.gpuMemTotalMb && sample.gpuMemTotalMb > 0
          ? Math.min(100, (sample.gpuMemUsedMb / sample.gpuMemTotalMb) * 100)
          : null;
    this.charts.vram.data.push(vramPct);

    atualizarRing("dp-cpu-ring", "dp-cpu-ring-num", sample.cpuPercent);
    setText("dp-cpu-frequencia", sample.cpuFreqAvgMhz == null ? null : `${sample.cpuFreqAvgMhz} MHz`);
    setText("dp-gpu-memoria", sample.gpuMemUsedMb == null ? null : `${(sample.gpuMemUsedMb / 1024).toFixed(1)} GiB`);
    setText("dp-gpu-temperatura", sample.gpuTempC == null ? null : `${sample.gpuTempC} °C`);
    const swapBar = document.querySelector<HTMLElement>("#dp-swap-bar .bar__fill");
    if (swapBar) {
      swapBar.style.width =
        sample.swapUsedPercent != null ? `${Math.min(100, sample.swapUsedPercent).toFixed(1)}%` : "0%";
    }
    setText("dp-swap-uso",
      sample.swapUsedPercent == null
        ? null
        : sample.swapUsedPercent > 0
          ? `${sample.swapUsedPercent.toFixed(0)}% em uso`
          : "Sem swap ativo");

    const ramTotal = document.getElementById("dp-ram-total");
    if (ramTotal && sample.memTotalKb != null) {
      ramTotal.textContent =
        sample.memUsedKb != null
          ? `${gib(sample.memUsedKb)} / ${gib(sample.memTotalKb)}`
          : `— / ${gib(sample.memTotalKb)}`;
    }
    const ramBarFill = document.querySelector<HTMLElement>("#dp-ram-bar .bar__fill");
    if (ramBarFill && sample.memUsedPercent != null) {
      ramBarFill.style.width = `${Math.max(0, Math.min(100, sample.memUsedPercent)).toFixed(1)}%`;
    }
    setText("dp-ram-pct", sample.memUsedPercent == null ? null : `${sample.memUsedPercent.toFixed(0)}% em uso`);
    setText(
      "dp-ram-disponivel",
      sample.memUsedKb == null || sample.memTotalKb == null
        ? null
        : gib(Math.max(0, sample.memTotalKb - sample.memUsedKb)),
    );

    Object.values(this.charts).forEach((c) => this.renderChart(c));
  }
}

function setText(id: string, value: string | null): void {
  const el = document.getElementById(id);
  if (!el) return;
  el.textContent = value ?? "Não disponível";
}
