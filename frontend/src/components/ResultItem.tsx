import type { Summary } from "@/lib/api";

interface Props {
  index: number;
  item: Summary;
  selected?: boolean;
  onSelect: (pmid: string) => void;
}

export function ResultItem({ index, item, selected, onSelect }: Props) {
  const year = (item.pubdate || "").split(/\s|;/)[0] || item.pubdate;

  return (
    <article
      className={`grid cursor-pointer grid-cols-[40px_minmax(0,1fr)] gap-3 border-b border-paper-rule/60 py-3 last:border-b-0 ${
        selected ? "bg-paper-dark/30" : "hover:bg-paper-dark/15"
      }`}
      onClick={() => onSelect(item.pmid)}
    >
      <div className="select-none pt-0.5 text-right font-serif text-xl font-bold leading-none text-paper-rule">
        {index}
      </div>

      <div className="min-w-0">
        <h3
          className="font-serif text-[16px] font-semibold leading-snug text-paper-ink hover:underline"
          dangerouslySetInnerHTML={{ __html: item.title }}
        />

        <p className="mt-0.5 text-[13px] text-paper-sepia">
          {item.authors.slice(0, 6).join(", ")}
          {item.authors.length > 6 && ", et al."}
        </p>

        <p className="text-[11px] text-paper-brown">
          <span className="italic">{item.source}</span>
          {year && <> · {year}</>}
          {item.doi && <> · doi:{item.doi}</>}
        </p>

        {item.abstract_text && (
          <p className="mt-1.5 line-clamp-3 text-[13px] leading-snug text-paper-ink/80">
            {item.abstract_text}
          </p>
        )}
      </div>
    </article>
  );
}
