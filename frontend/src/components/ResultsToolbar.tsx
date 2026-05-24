import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface Props {
  total: number;
  query: string;
  sort: string;
  onSortChange: (s: string) => void;
  pageSize: number;
  onPageSizeChange: (n: number) => void;
  display: string;
  onDisplayChange: (d: string) => void;
}

export function ResultsToolbar({
  total,
  query,
  sort,
  onSortChange,
  pageSize,
  onPageSizeChange,
  display,
  onDisplayChange,
}: Props) {
  return (
    <div className="flex flex-col gap-2 border-b border-paper-rule/60 pb-3 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0">
        <p className="font-serif text-2xl font-bold tracking-tight text-paper-ink">
          {total.toLocaleString()}{" "}
          <span className="font-serif text-sm font-normal italic text-paper-brown">
            dispatches
          </span>
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
      </div>
    </div>
  );
}
