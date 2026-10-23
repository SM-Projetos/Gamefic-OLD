const DPR = typeof devicePixelRatio === "number" && devicePixelRatio > 0 ? devicePixelRatio : 1;

export interface ChartStyle {
  stroke: string;
  fill: string;
  grid?: string;
}

export class RollingData {
  private points: (number | null)[] = [];

  constructor(private readonly limit: number) {}

  push(value: number | null): void {
    this.points.push(value);
    while (this.points.length > this.limit) this.points.shift();
  }

  values(): readonly (number | null)[] {
    return this.points;
  }

  last(): number | null {
    for (let i = this.points.length - 1; i >= 0; i -= 1) {
      const v = this.points[i];
      if (v != null) return v;
    }
    return null;
  }
}

export function drawChart(
  canvas: HTMLCanvasElement,
  data: readonly (number | null)[],
  maxY: number,
  style: ChartStyle,
  unit: string,
  clear = true,
): void {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  if (width === 0 || height === 0) return;

  canvas.width = Math.round(width * DPR);
  canvas.height = Math.round(height * DPR);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
  if (clear) ctx.clearRect(0, 0, width, height);

  const padL = 4;
  const padB = 4;
  const plotW = Math.max(1, width - padL);
  const plotH = Math.max(1, height - padB);

  const grid = style.grid ?? "rgba(128, 132, 148, 0.22)";
  if (clear) {
    ctx.strokeStyle = grid;
    ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i += 1) {
      const y = (plotH / 4) * i + 0.5;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
  }

  if (data.length < 2 || data.every((v) => v == null)) {
    ctx.fillStyle = "var(--text-dim)".startsWith("var") ? "#9aa0b4" : "#9aa0b4";
    ctx.font = "13px 'Inter', sans-serif";
    ctx.textAlign = "center";
    ctx.fillText("sem dados", width / 2, height / 2);
    return;
  }

  const stepX = plotW / (data.length - 1);
  const yOf = (v: number) => plotH - (Math.min(Math.max(v, 0), maxY) / maxY) * plotH;

  const segments: Array<Array<[number, number]>> = [];
  let current: Array<[number, number]> = [];

  data.forEach((v, i) => {
    if (v == null) {
      if (current.length > 1) segments.push(current);
      current = [];
      return;
    }
    current.push([i * stepX + padL, yOf(v)]);
  });
  if (current.length > 1) segments.push(current);

  segments.forEach((segment) => {
    ctx.beginPath();
    segment.forEach(([x, y], i) => {
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });

    ctx.strokeStyle = style.stroke;
    ctx.lineWidth = 1.8;
    ctx.lineJoin = "round";
    ctx.lineCap = "round";
    ctx.stroke();

    const last = segment[segment.length - 1];
    ctx.lineTo(last[0], plotH);
    ctx.lineTo(segment[0][0], plotH);
    ctx.closePath();
    ctx.fillStyle = style.fill;
    ctx.fill();

    ctx.beginPath();
    ctx.arc(last[0], last[1], 2.6, 0, Math.PI * 2);
    ctx.fillStyle = style.stroke;
    ctx.fill();
  });

  ctx.fillStyle = "#9aa0b4";
  ctx.font = "11px 'JetBrains Mono', monospace";
  ctx.textAlign = "left";
  ctx.fillText(`${maxY}${unit}`, 2, 10);
}
