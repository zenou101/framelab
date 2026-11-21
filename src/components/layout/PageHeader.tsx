import type { ReactNode } from "react";

export function PageHeader({
  title,
  description,
  actions,
  eyebrow,
}: {
  title: string;
  description?: ReactNode;
  actions?: ReactNode;
  eyebrow?: string;
}) {
  return (
    <div className="flex items-start justify-between gap-6 border-b border-line px-6 py-5">
      <div className="min-w-0">
        {eyebrow ? (
          <div className="mb-1 text-[10.5px] font-medium tracking-[0.07em] text-text-3 uppercase">{eyebrow}</div>
        ) : null}
        <h1 className="text-[19px] leading-tight font-medium tracking-[-0.014em] text-text-1">{title}</h1>
        {description ? <p className="mt-1 max-w-[720px] text-[12.5px] leading-relaxed text-text-2">{description}</p> : null}
      </div>
      {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
    </div>
  );
}

export function PageBody({ children }: { children: ReactNode }) {
  return <div className="space-y-4 px-6 py-5">{children}</div>;
}
