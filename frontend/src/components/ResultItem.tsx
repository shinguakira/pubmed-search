import { Link } from "react-router-dom";
import type { Summary } from "@/lib/api";
import { useSaved } from "@/hooks/useSaved";

interface Props {
  index: number;
  item: Summary;
  display?: string;
  onCite: (pmid: string) => void;
}

const HIGHLIGHT_TYPES = [
  "Review",
  "Systematic Review",
  "Meta-Analysis",
  "Clinical Trial",
  "Randomized Controlled Trial",
  "Case Reports",
];

export function ResultItem({ index, item, display = "summary", onCite }: Props) {
  const { has, toggle } = useSaved();
  const saved = has(item.pmid);

  if (display === "pmid") {
    return (
      <div className="flex items-baseline gap-3 border-b border-paper-rule/60 py-2 font-mono text-xs text-paper-ink">
        <span className="w-8 shrink-0 text-right text-paper-fade">{index}</span>
        <Link
          to={`/article/${item.pmid}`}
          className="text-paper-rust hover:underline"
        >
          {item.pmid}
        </Link>
        <span className="truncate text-paper-sepia">{item.title}</span>
      </div>
    );
  }

  const primaryType = item.pubtypes.find((t) => HIGHLIGHT_TYPES.includes(t));
  const year = (item.pubdate || "").split(/\s|;/)[0] || item.pubdate;

  return (
    <article className="group relative grid grid-cols-[44px_minmax(0,1fr)] gap-3 border-b border-double border-paper-rule/70 py-3 last:border-b-0">
      <div className="select-none pt-0.5 text-right font-serif text-2xl font-bold leading-none text-paper-rule">
        {String(index).padStart(2, "0")}
      </div>

      <div className="min-w-0">
        <div className="mb-0.5 flex items-baseline justify-between gap-3">
          <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-fade">
            PMID&nbsp;{item.pmid}
          </span>
          {primaryType && (
            <span className="border border-paper-rust/40 bg-paper-light/60 px-1.5 py-0 font-mono text-[10px] uppercase tracking-[0.16em] text-paper-rust">
              {primaryType}
            </span>
          )}
        </div>

        <h3 className="font-serif text-[17px] font-semibold leading-snug tracking-tight text-paper-ink">
          <Link
            to={`/article/${item.pmid}`}
            className="decoration-paper-rust/40 underline-offset-4 visited:text-paper-sepia hover:underline"
            dangerouslySetInnerHTML={{ __html: item.title }}
          />
        </h3>

        <p className="mt-0.5 font-serif text-[13px] italic text-paper-sepia">
          By{" "}
          {item.authors.slice(0, 8).join(", ")}
          {item.authors.length > 8 && ", et al."}
        </p>

        <p className="mt-0.5 font-mono text-[10px] uppercase tracking-[0.14em] text-paper-brown">
          <span className="not-italic">{item.source}</span>
          {year && <> &nbsp;·&nbsp; {year}</>}
          {item.volume && (
            <>
              {" "}
              &nbsp;·&nbsp; Vol.&nbsp;{item.volume}
              {item.issue && `(${item.issue})`}
            </>
          )}
          {item.pages && <> &nbsp;·&nbsp; pp.&nbsp;{item.pages}</>}
          {item.doi && (
            <>
              {" "}
              &nbsp;·&nbsp; DOI&nbsp;{item.doi}
            </>
          )}
        </p>

        {item.abstract_text && (
          <p className="mt-1.5 line-clamp-3 font-serif text-[13px] leading-snug text-paper-ink/80">
            {item.abstract_text}
          </p>
        )}

        <div className="mt-1.5 flex flex-wrap items-center gap-x-4 gap-y-0 font-serif text-[12px] text-paper-rust">
          <button
            type="button"
            onClick={() => onCite(item.pmid)}
            className="hover:underline"
          >
            ▸ Cite
          </button>
          <button
            type="button"
            onClick={() => toggle(item)}
            className="hover:underline"
          >
            ▸ {saved ? "Saved" : "Save"}
          </button>
          {item.doi && (
            <a
              href={`https://doi.org/${item.doi}`}
              target="_blank"
              rel="noreferrer"
              className="hover:underline"
            >
              ▸ DOI
            </a>
          )}
          <a
            href={`https://pubmed.ncbi.nlm.nih.gov/${item.pmid}/`}
            target="_blank"
            rel="noreferrer"
            className="hover:underline"
          >
            ▸ PubMed
          </a>
        </div>
      </div>
    </article>
  );
}
