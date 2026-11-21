import { Fragment, type ReactNode } from "react";

/**
 * Deliberately small markdown renderer for the bundled legal documents.
 *
 * Only the constructs those documents actually use are supported: headings,
 * paragraphs, lists, fenced code, blockquotes, rules, tables, bold, inline code
 * and links. Content is authored in this repository, never user-supplied, so no
 * HTML is accepted or rendered.
 */

const INLINE = /(\*\*[^*]+\*\*|\*[^*\n]+\*|`[^`]+`|\[[^\]]+\]\([^)]+\)|<https?:\/\/[^>\s]+>|https?:\/\/[^\s)]+)/g;

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  const parts = text.split(INLINE).filter((part) => part !== "");
  return parts.map((part, index) => {
    const key = `${keyPrefix}-${index}`;
    if (part.startsWith("**") && part.endsWith("**")) {
      return (
        <strong key={key} className="font-medium text-text-1">
          {part.slice(2, -2)}
        </strong>
      );
    }
    if (part.startsWith("*") && part.endsWith("*") && part.length > 2) {
      return (
        <em key={key} className="italic">
          {part.slice(1, -1)}
        </em>
      );
    }
    if (part.startsWith("`") && part.endsWith("`")) {
      return (
        <code key={key} className="rounded-[4px] bg-[color-mix(in_oklab,var(--text-1)_7%,transparent)] px-1 py-0.5 font-mono text-[11px] text-text-1">
          {part.slice(1, -1)}
        </code>
      );
    }
    const link = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(part);
    if (link) {
      const [, label, href] = link;
      const external = /^https?:/.test(href);
      return (
        <a
          key={key}
          href={href}
          {...(external ? { target: "_blank", rel: "noreferrer noopener" } : {})}
          className="text-accent underline decoration-[color-mix(in_oklab,var(--accent)_45%,transparent)] underline-offset-2 hover:decoration-accent"
        >
          {label}
        </a>
      );
    }
    if (part.startsWith("<http") && part.endsWith(">")) {
      const href = part.slice(1, -1);
      return (
        <a key={key} href={href} target="_blank" rel="noreferrer noopener" className="text-accent underline underline-offset-2">
          {href}
        </a>
      );
    }
    if (/^https?:\/\//.test(part)) {
      return (
        <a key={key} href={part} target="_blank" rel="noreferrer noopener" className="text-accent underline underline-offset-2">
          {part}
        </a>
      );
    }
    return <Fragment key={key}>{part}</Fragment>;
  });
}

/** Joins a list item's wrapped continuation lines (indented, no marker). */
function consumeItem(lines: string[], start: number, first: string): [string, number] {
  let item = first;
  let index = start;
  while (
    index < lines.length &&
    lines[index].trim() !== "" &&
    /^\s{2,}\S/.test(lines[index]) &&
    !/^\s*([-*]|\d+\.)\s+/.test(lines[index])
  ) {
    item = `${item} ${lines[index].trim()}`;
    index += 1;
  }
  return [item, index];
}

export function Markdown({ text }: { text: string }) {
  const lines = text.split(/\r?\n/);
  const blocks: ReactNode[] = [];
  let index = 0;

  const flushParagraph = (buffer: string[], key: string) => {
    if (buffer.length === 0) return;
    blocks.push(
      <p key={key} className="text-[12.5px] leading-relaxed text-text-2">
        {renderInline(buffer.join(" "), key)}
      </p>,
    );
  };

  while (index < lines.length) {
    const line = lines[index];
    const key = `b-${index}`;

    if (line.trimStart().startsWith("```")) {
      index += 1;
      const code: string[] = [];
      while (index < lines.length && !lines[index].trimStart().startsWith("```")) {
        code.push(lines[index]);
        index += 1;
      }
      index += 1;
      blocks.push(
        <pre
          key={key}
          className="overflow-x-auto rounded-[8px] border border-line bg-[color-mix(in_oklab,var(--text-1)_3%,transparent)] px-3 py-2 font-mono text-[10.5px] leading-[1.55] whitespace-pre-wrap text-text-2"
        >
          {code.join("\n")}
        </pre>,
      );
      continue;
    }

    const heading = /^(#{1,4})\s+(.*)$/.exec(line);
    if (heading) {
      const level = heading[1].length;
      const content = renderInline(heading[2], key);
      const className =
        level <= 2
          ? "mt-2 text-[14px] font-medium tracking-[-0.01em] text-text-1"
          : "mt-1 text-[12.5px] font-medium text-text-1";
      blocks.push(
        level <= 2 ? (
          <h2 key={key} className={className}>
            {content}
          </h2>
        ) : (
          <h3 key={key} className={className}>
            {content}
          </h3>
        ),
      );
      index += 1;
      continue;
    }

    if (/^\s*\|.+\|\s*$/.test(line) && /^\s*\|[\s:|-]+\|\s*$/.test(lines[index + 1] ?? "")) {
      const readRow = (row: string) =>
        row.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map((cell) => cell.trim());
      const header = readRow(line);
      index += 2;
      const rows: string[][] = [];
      while (index < lines.length && /^\s*\|.+\|\s*$/.test(lines[index])) {
        rows.push(readRow(lines[index]));
        index += 1;
      }
      blocks.push(
        <div key={key} className="overflow-x-auto rounded-[8px] border border-line">
          <table className="w-full border-collapse text-[11.5px]">
            <thead>
              <tr className="border-b border-line bg-[color-mix(in_oklab,var(--text-1)_3%,transparent)] text-left">
                {header.map((cell, cellIndex) => (
                  <th key={cellIndex} className="px-2.5 py-1.5 font-medium text-text-2">
                    {renderInline(cell, `${key}-h-${cellIndex}`)}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row, rowIndex) => (
                <tr key={rowIndex} className="border-b border-line last:border-0">
                  {row.map((cell, cellIndex) => (
                    <td key={cellIndex} className="px-2.5 py-1.5 align-top text-text-2">
                      {renderInline(cell, `${key}-r${rowIndex}-${cellIndex}`)}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>,
      );
      continue;
    }

    if (/^\s*[-*]\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*[-*]\s+/.test(lines[index])) {
        const [item, next] = consumeItem(lines, index + 1, lines[index].replace(/^\s*[-*]\s+/, ""));
        items.push(item);
        index = next;
      }
      blocks.push(
        <ul key={key} className="ml-4 list-disc space-y-1 text-[12.5px] leading-relaxed text-text-2 marker:text-text-3">
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>{renderInline(item, `${key}-i-${itemIndex}`)}</li>
          ))}
        </ul>,
      );
      continue;
    }

    if (/^\s*\d+\.\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*\d+\.\s+/.test(lines[index])) {
        const [item, next] = consumeItem(lines, index + 1, lines[index].replace(/^\s*\d+\.\s+/, ""));
        items.push(item);
        index = next;
      }
      blocks.push(
        <ol key={key} className="ml-4 list-decimal space-y-1 text-[12.5px] leading-relaxed text-text-2 marker:text-text-3">
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>{renderInline(item, `${key}-o-${itemIndex}`)}</li>
          ))}
        </ol>,
      );
      continue;
    }

    if (/^\s*>\s?/.test(line)) {
      const quoted: string[] = [];
      while (index < lines.length && /^\s*>\s?/.test(lines[index])) {
        quoted.push(lines[index].replace(/^\s*>\s?/, ""));
        index += 1;
      }
      blocks.push(
        <blockquote key={key} className="border-l-2 border-line-strong pl-3 text-[12px] leading-relaxed text-text-3">
          {renderInline(quoted.join(" "), key)}
        </blockquote>,
      );
      continue;
    }

    if (/^\s*(---+|\*\*\*+)\s*$/.test(line)) {
      blocks.push(<hr key={key} className="border-line" />);
      index += 1;
      continue;
    }

    if (line.trim() === "") {
      index += 1;
      continue;
    }

    const paragraph: string[] = [];
    while (
      index < lines.length &&
      lines[index].trim() !== "" &&
      !/^(#{1,4}\s|```|\s*[-*]\s|\s*\d+\.\s|\s*>\s?|\s*\|)/.test(lines[index])
    ) {
      paragraph.push(lines[index].trim());
      index += 1;
    }
    // A line starting with a marker that did not form a block (for example a
    // pipe character without a table separator) is treated as plain text so the
    // renderer can never stall on it.
    if (paragraph.length === 0) {
      paragraph.push(line.trim());
      index += 1;
    }
    flushParagraph(paragraph, key);
  }

  return <div className="space-y-2.5">{blocks}</div>;
}
