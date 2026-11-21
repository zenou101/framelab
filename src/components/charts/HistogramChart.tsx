import { Bar, BarChart, CartesianGrid, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import type { HistogramBin } from "@/lib/types";

export default function HistogramChart({ bins, height = 180 }: { bins: HistogramBin[]; height?: number }) {
  if (bins.length === 0) {
    return (
      <div style={{ height }} className="grid place-items-center text-[12px] text-text-3">
        No distribution to show
      </div>
    );
  }
  const data = bins.map((bin) => ({ label: ((bin.fromMs + bin.toMs) / 2).toFixed(1), count: bin.count }));
  return (
    <div style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={data} margin={{ top: 6, right: 8, bottom: 0, left: -20 }}>
          <CartesianGrid stroke="var(--grid)" vertical={false} />
          <XAxis dataKey="label" tick={{ fill: "var(--text-3)", fontSize: 10 }} tickLine={false} axisLine={{ stroke: "var(--grid)" }} minTickGap={24} unit="ms" />
          <YAxis tick={{ fill: "var(--text-3)", fontSize: 10 }} tickLine={false} axisLine={false} width={44} />
          <Tooltip
            cursor={{ fill: "var(--accent-soft)" }}
            contentStyle={{ background: "var(--elevated)", border: "1px solid var(--border)", borderRadius: 8, fontSize: 11.5 }}
            formatter={(value) => [`${value} frames`, "Count"]}
            labelFormatter={(label) => `${label} ms`}
          />
          <Bar dataKey="count" fill="var(--series-1)" radius={[2, 2, 0, 0]} isAnimationActive={false} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
