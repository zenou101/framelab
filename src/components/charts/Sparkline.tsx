export default function Sparkline({
  values,
  height = 34,
  max,
  className,
}: {
  values: number[];
  height?: number;
  max?: number;
  className?: string;
}) {
  if (values.length < 2) {
    return <div style={{ height }} className={className} aria-hidden />;
  }
  const ceiling = max ?? Math.max(...values, 1);
  const step = 100 / (values.length - 1);
  const points = values
    .map((value, index) => `${(index * step).toFixed(2)},${(100 - (value / ceiling) * 100).toFixed(2)}`)
    .join(" ");
  const area = `0,100 ${points} 100,100`;
  return (
    <svg
      viewBox="0 0 100 100"
      preserveAspectRatio="none"
      style={{ height }}
      className={className}
      aria-hidden
    >
      <polygon points={area} fill="var(--accent)" opacity="0.1" />
      <polyline
        points={points}
        fill="none"
        stroke="var(--accent)"
        strokeWidth="1.6"
        vectorEffect="non-scaling-stroke"
        strokeLinejoin="round"
        strokeLinecap="round"
      />
    </svg>
  );
}
