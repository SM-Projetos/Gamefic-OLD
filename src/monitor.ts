import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import { drawChart, RollingData } from "./charts";
import { gib } from "./system";

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
  gpuTempC: number | null;
}

const HISTORY = 300;

interface ChartSet {
  cpu: ChartCard;
  ram: ChartCard;
  gpu: ChartCard;
  temp: ChartCard;
}

interface ChartCard {
  canvas: HTMLCanvasElement | null;
  data: RollingData;
  render: () => void;
}

export class Monitor {
  private charts: ChartSet;
  private unlisten: UnlistenFn | null = null;
  private intervalMs = 2000;
  private resizeObserver: ResizeObserver | null = null;

  constructor(
    private readonly onChangeInterval: (intervalMs: number) => void,
  ) {
    this.charts = this.setupCharts();
  }

  private setupCharts(): ChartSet {
    return {
      cpu: makeChart("grafico-cpu", "#a78bfa", "rgba(167, 139, 250, 0.14)", 100, "%"),
      ram: makeChart("grafico-ram", "#22d3ee", "rgba(34, 211, 238, 0.12)", 100, "%"),
      gpu: makeChart("grafico-gpu", "#34d399", "rgba(52, 211, 153, 0.12)", 100, "%"),
      temp: makeChart("grafico-temp", "#fbbf24", "rgba(251, 191, 36, 0.12)", 100, "°C"),
    };
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
  }  setInterval(intervalMs: number): void {
    this.intervalMs = intervalMs;
    this.onChangeInterval(intervalMs);
  }

  hide(): void {
    this.resizeObserver?.disconnect();
  }

  show(canvasHost: HTMLElement | null = null): void {
    if (canvasHost) {
      this.resizeObserver = new ResizeObserver(() => this.render());
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
    this.charts.cpu.render();
    this.charts.ram.render();
    this.charts.gpu.render();
    this.charts.temp.render();
  }

  private pushSample(sample: MetricSample): void {
    this.charts.cpu.data.push(sample.cpuPercent);
    this.charts.ram.data.push(sample.memUsedPercent);
    this.charts.gpu.data.push(sample.gpuUtilizationPercent);
    this.charts.temp.data.push(sample.tempMaxC);

    setText("num-cpu-percento", porcento(sample.cpuPercent));
    setText("num-cpu-frequencia", sample.cpuFreqAvgMhz == null ? null : `${sample.cpuFreqAvgMhz} MHz`);
    setText("num-ram-uso", `${gib(sample.memUsedKb)} de ${gib(sample.memTotalKb)}`);
    setText("num-swap-uso", porcento(sample.swapUsedPercent));
    setText("num-gpu-percento", porcento(sample.gpuUtilizationPercent));
    setText(
      "num-gpu-mem",
      sample.gpuMemUsedMb == null ? null : `${(sample.gpuMemUsedMb / 1024).toFixed(1)} GiB`,
    );
    setText("num-gpu-temp", sample.gpuTempC == null ? null : `${sample.gpuTempC} °C`);
    setText("num-temp-maxima", sample.tempMaxC == null ? null : `${sample.tempMaxC.toFixed(1)} °C`);

    this.charts.cpu.render();
    this.charts.ram.render();
    this.charts.gpu.render();
    this.charts.temp.render();
  }
}

function makeChart(
  canvasId: string,
  stroke: string,
  fill: string,
  maxY: number,
  unit: string,
): ChartCard {
  const data = new RollingData(HISTORY);
  const canvas = document.getElementById(canvasId) as HTMLCanvasElement | null;

  return {
    canvas,
    data,
    render: () => {
      if (canvas && canvas.isConnected) {
        drawChart(canvas, data.values(), maxY, { stroke, fill }, unit);
      }
    },
  };
}

function porcento(v: number | null | undefined): string | null {
  return v == null ? null : `${v.toFixed(0)}%`;
}

function setText(id: string, value: string | null): void {
  const el = document.getElementById(id);
  if (!el) return;
  el.textContent = value ?? "Não disponível";
}
