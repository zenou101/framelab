import { CartesianGrid, Legend, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { SampleSeries } from "@/lib/types";

export default function SampleChart({ series, height = 200 }: { series: SampleSeries; height?: number }) {
  const data = series.tMs.map((t, index) => ({
    t: Math.round((t - series.tMs[0]) / 1000),
    cpu: series.cpu[index] ?? null,
    gpu: series.gpu[index] ?? null,
    ram: series.ram[index] ?? null,
  }));
  if (data.length < 2) {
    return (
      <div style={{ height }} className="grid place-items-center text-[12px] text-text-3">
        No system samples were stored with this session
      </div>
    );
  }
  return (
    <div style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data} margin={{ top: 8, right: 8, bottom: 0, left: -18 }}>
          <CartesianGrid stroke="var(--grid)" strokeDasharray="2 4" vertical={false} />
          <XAxis dataKey="t" tick={{ fill: "var(--text-3)", fontSize: 10 }} tickLine={false} axisLine={{ stroke: "var(--grid)" }} unit="s" minTickGap={40} />
          <YAxis domain={[0, 100]} tick={{ fill: "var(--text-3)", fontSize: 10 }} tickLine={false} axisLine={false} width={44} unit="%" />
          <Tooltip contentStyle={{ background: "var(--elevated)", border: "1px solid var(--border)", borderRadius: 8, fontSize: 11.5 }} />
          <Legend wrapperStyle={{ fontSize: 11, color: "var(--text-3)" }} />
          <Line type="monotone" dataKey="cpu" name="CPU" stroke="var(--series-1)" strokeWidth={1.4} dot={false} isAnimationActive={false} />
          <Line type="monotone" dataKey="gpu" name="GPU" stroke="var(--series-2)" strokeWidth={1.4} dot={false} isAnimationActive={false} />
          <Line type="monotone" dataKey="ram" name="RAM" stroke="var(--text-3)" strokeWidth={1.2} strokeDasharray="3 3" dot={false} isAnimationActive={false} />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
