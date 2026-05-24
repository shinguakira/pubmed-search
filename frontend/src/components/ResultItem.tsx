import { Link } from "react-router-dom";
import { BookmarkIcon, BookmarkCheck, Quote, ExternalLink } from "lucide-react";
import type { Summary } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useSaved } from "@/hooks/useSaved";
import { cn } from "@/lib/utils";

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
      <div className="flex items-baseline gap-3 border-b py-2 font-mono text-xs">
        <span className="w-8 shrink-0 text-right text-muted-foreground">{index}</span>
        <Link to={`/article/${item.pmid}`} className="text-pubmed hover:underline">
          {item.pmid}
        </Link>
        <span className="truncate text-foreground/70">{item.title}</span>
      </div>
    );
  }

  const primaryType = item.pubtypes.find((t) => HIGHLIGHT_TYPES.includes(t));
  const year = (item.pubdate || "").split(/\s|;/)[0] || item.pubdate;

  return (
    <article
      className={cn(
        "group relative flex h-full flex-col overflow-hidden rounded-xl border bg-card p-4",
        "shadow-sm transition-all hover:-translate-y-0.5 hover:border-pubmed/40 hover:shadow-md",
      )}
    >
      <header className="mb-2 flex items-start justify-between gap-2">
        <span className="font-mono text-[10px] text-muted-foreground">
          #{index}
        </span>
        <div className="flex flex-wrap items-center justify-end gap-1">
          {primaryType && (
            <Badge variant="soft" className="text-[10px]">
              {primaryType}
            </Badge>
          )}
        </div>
      </header>

      <Link
        to={`/article/${item.pmid}`}
        className="line-clamp-3 text-[14px] font-medium leading-snug text-pubmed visited:text-purple-800 hover:underline"
        title={item.title}
        dangerouslySetInnerHTML={{ __html: item.title }}
      />

      <p className="mt-2 line-clamp-1 text-xs text-foreground/75">
        {item.authors.slice(0, 5).join(", ")}
        {item.authors.length > 5 && ", et al."}
      </p>

      <p className="line-clamp-1 text-xs italic text-muted-foreground">
        {item.source}
        {year && ` · ${year}`}
      </p>

      <div className="mt-auto pt-3">
        <div className="flex items-center justify-between">
          <Badge variant="outline" className="font-mono text-[10px]">
            PMID {item.pmid}
          </Badge>
          <div className="flex items-center gap-0.5 opacity-70 transition-opacity group-hover:opacity-100">
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={() => onCite(item.pmid)}
              title="Cite"
            >
              <Quote className="h-3.5 w-3.5" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={() => toggle(item)}
              title={saved ? "Saved" : "Save"}
            >
              {saved ? (
                <BookmarkCheck className="h-3.5 w-3.5 text-pubmed" />
              ) : (
                <BookmarkIcon className="h-3.5 w-3.5" />
              )}
            </Button>
            {item.doi && (
              <a
                href={`https://doi.org/${item.doi}`}
                target="_blank"
                rel="noreferrer"
                title="DOI"
                className="inline-flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground"
              >
                <ExternalLink className="h-3.5 w-3.5" />
              </a>
            )}
          </div>
        </div>
      </div>
    </article>
  );
}
