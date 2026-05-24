import { Link } from "react-router-dom";
import { BookmarkIcon, BookmarkCheck, Quote, ExternalLink } from "lucide-react";
import type { Summary } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useSaved } from "@/hooks/useSaved";

interface Props {
  index: number;
  item: Summary;
  display?: string;
  onCite: (pmid: string) => void;
}

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

  const primaryType = item.pubtypes.find((t) =>
    [
      "Review",
      "Systematic Review",
      "Meta-Analysis",
      "Clinical Trial",
      "Randomized Controlled Trial",
      "Case Reports",
    ].includes(t),
  );

  return (
    <article className="group flex gap-4 border-b py-5 transition-colors hover:bg-accent/30">
      <div className="w-8 shrink-0 pt-1 text-right text-xs font-medium text-muted-foreground">
        {index}
      </div>
      <div className="min-w-0 flex-1 space-y-1.5">
        <h3 className="text-[15px] font-medium leading-snug">
          <Link
            to={`/article/${item.pmid}`}
            className="text-pubmed visited:text-purple-700 hover:underline"
            dangerouslySetInnerHTML={{ __html: item.title }}
          />
        </h3>
        <p className="text-sm text-foreground/80">
          {item.authors.slice(0, 6).join(", ")}
          {item.authors.length > 6 && ", et al."}
        </p>
        <p className="text-xs text-muted-foreground">
          <span className="italic">{item.source}</span>
          {item.pubdate && <span>. {item.pubdate}</span>}
          {item.volume && (
            <span>
              ;{item.volume}
              {item.issue && `(${item.issue})`}
            </span>
          )}
          {item.pages && <span>:{item.pages}</span>}
          {item.doi && <span>. doi: {item.doi}</span>}
        </p>
        <div className="flex flex-wrap items-center gap-1.5 pt-1">
          <Badge variant="outline" className="font-mono text-[10px]">
            PMID: {item.pmid}
          </Badge>
          {primaryType && (
            <Badge variant="soft" className="text-[10px]">
              {primaryType}
            </Badge>
          )}
          {item.lang && item.lang !== "eng" && (
            <Badge variant="secondary" className="text-[10px] uppercase">
              {item.lang}
            </Badge>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-1 pt-2 opacity-60 transition-opacity group-hover:opacity-100">
          <Button
            variant="ghost"
            size="sm"
            className="h-8 px-2 text-xs"
            onClick={() => onCite(item.pmid)}
          >
            <Quote className="h-3.5 w-3.5" />
            Cite
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-8 px-2 text-xs"
            onClick={() => toggle(item)}
          >
            {saved ? (
              <BookmarkCheck className="h-3.5 w-3.5 text-pubmed" />
            ) : (
              <BookmarkIcon className="h-3.5 w-3.5" />
            )}
            {saved ? "Saved" : "Save"}
          </Button>
          {item.doi && (
            <a
              href={`https://doi.org/${item.doi}`}
              target="_blank"
              rel="noreferrer"
              className="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
            >
              <ExternalLink className="h-3.5 w-3.5" />
              DOI
            </a>
          )}
          <a
            href={`https://pubmed.ncbi.nlm.nih.gov/${item.pmid}/`}
            target="_blank"
            rel="noreferrer"
            className="inline-flex h-8 items-center gap-1 rounded-md px-2 text-xs text-muted-foreground hover:bg-accent hover:text-foreground"
          >
            <ExternalLink className="h-3.5 w-3.5" />
            PubMed
          </a>
        </div>
      </div>
    </article>
  );
}
