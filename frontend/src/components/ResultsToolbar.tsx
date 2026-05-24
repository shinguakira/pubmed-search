import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";

interface Props {
  total: number;
  query: string;
  elapsedMs?: number;
  sort: string;
  onSortChange: (s: string) => void;
  pageSize: number;
  onPageSizeChange: (n: number) => void;
  display: string;
  onDisplayChange: (d: string) => void;
  /// When true the search uses esearch+efetch_bulk and prewarms the
  /// per-PMID article cache so detail clicks are instant.
  bulk: boolean;
  onBulkChange: (b: boolean) => void;
}

export function ResultsToolbar({
  total,
  query,
  elapsedMs,
  sort,
  onSortChange,
  pageSize,
  onPageSizeChange,
  display,
  onDisplayChange,
  bulk,
  onBulkChange,
}: Props) {
  return (
    <div className="flex flex-col gap-2 border-b border-paper-rule/60 pb-3 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0">
        <p
          className="font-serif text-2xl font-bold tracking-tight text-paper-ink"
          data-testid="result-count"
        >
          {total.toLocaleString()}{" "}
          <span className="font-serif text-sm font-normal italic text-paper-brown">
            dispatches
          </span>
          {typeof elapsedMs === "number" && (
            <span
              className="ml-2 font-mono text-[11px] font-normal uppercase tracking-[0.14em] text-paper-fade"
              data-testid="elapsed-time"
              title={`Backend reply in ${elapsedMs} ms`}
            >
              · {(elapsedMs / 1000).toFixed(2)}s
            </span>
          )}
        </p>
        {query && (
          <p className="truncate font-mono text-[10px] uppercase tracking-[0.14em] text-paper-fade">
            for <span className="text-paper-sepia">{query}</span>
          </p>
        )}
      </div>
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-brown">Display</span>
          <Select value={display} onValueChange={onDisplayChange}>
            <SelectTrigger className="h-9 w-[120px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="summary">Summary</SelectItem>
              <SelectItem value="abstract">Abstract</SelectItem>
              <SelectItem value="pmid">PMID</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-brown">Sort by</span>
          <Select value={sort} onValueChange={onSortChange}>
            <SelectTrigger className="h-9 w-[160px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="relevance">Best match</SelectItem>
              <SelectItem value="pub_date">Most recent</SelectItem>
              <SelectItem value="first_author">First author</SelectItem>
              <SelectItem value="journal">Journal</SelectItem>
              <SelectItem value="title">Title</SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-brown">Per page</span>
          <Select
            value={String(pageSize)}
            onValueChange={(v) => onPageSizeChange(Number(v))}
          >
            <SelectTrigger className="h-9 w-[80px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {[10, 20, 50, 100].map((n) => (
                <SelectItem key={n} value={String(n)}>
                  {n}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex items-center gap-2">
          <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-brown">
            Fetch
          </span>
          <div className="flex overflow-hidden rounded border border-paper-rule font-mono text-[10px] uppercase tracking-[0.14em]">
            <button
              type="button"
              onClick={() => onBulkChange(false)}
              className={cn(
                "px-2.5 py-1.5 transition-colors",
                !bulk
                  ? "bg-paper-ink text-paper-light"
                  : "bg-paper text-paper-brown hover:bg-paper-dark",
              )}
              title="esearch + esummary (light, no abstract)"
            >
              Default
            </button>
            <button
              type="button"
              onClick={() => onBulkChange(true)}
              className={cn(
                "px-2.5 py-1.5 transition-colors",
                bulk
                  ? "bg-paper-rust text-paper-light"
                  : "bg-paper text-paper-brown hover:bg-paper-dark",
              )}
              title="esearch(usehistory) + efetch_bulk — heavier per call, but article-detail clicks become instant"
            >
              Bulk
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
