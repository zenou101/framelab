import { Area, AreaChart, CartesianGrid, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { ms as fmtMs } from "@/lib/format";

export default function FrameChart({
  frames,
  p95,
  p99,
  height = 220,
}: {
  frames: number[];
  p95?: number | null;
  p99?: number | null;
  height?: number;
}) {
  if (frames.length < 2) {
    return (
      <div style={{ height }} className="grid place-items-center text-[12px] text-text-3">
        No frames captured yet
      </div>
    );
  }
  const data = frames.map((frame, index) => ({ index, frame }));
  return (
    <div style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data} margin={{ top: 8, right: 8, bottom: 0, left: -18 }}>
          <CartesianGrid stroke="var(--grid)" strokeDasharray="2 4" vertical={false} />
          <XAxis
            dataKey="index"
            tick={{ fill: "var(--text-3)", fontSize: 10 }}
            tickLine={false}
            axisLine={{ stroke: "var(--grid)" }}
            minTickGap={48}
          />
          <YAxis
            tick={{ fill: "var(--text-3)", fontSize: 10 }}
            tickLine={false}
            axisLine={false}
            width={48}
            unit="ms"
          />
          <Tooltip
            contentStyle={{
              background: "var(--elevated)",
              border: "1px solid var(--border)",
              borderRadius: 8,
              fontSize: 11.5,
              color: "var(--text-1)",
            }}
            formatter={(value) => [fmtMs(Number(value)), "Frame time"]}
            labelFormatter={(label) => `Frame ${label}`}
          />
          {p95 ? <ReferenceLine y={p95} stroke="var(--warn)" strokeDasharray="3 4" strokeOpacity={0.7} /> : null}
          {p99 ? <ReferenceLine y={p99} stroke="var(--err)" strokeDasharray="3 4" strokeOpacity={0.7} /> : null}
          <Area
            type="stepAfter"
            dataKey="frame"
            stroke="var(--series-1)"
            strokeWidth={1.4}
            fill="var(--series-1)"
            fillOpacity={0.09}
            isAnimationActive={false}
            dot={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
