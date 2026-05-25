import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface Props {
  total: number;
  elapsedMs?: number;
  sort: string;
  onSortChange: (s: string) => void;
  pageSize: number;
  onPageSizeChange: (n: number) => void;
}

export function ResultsToolbar({
  total,
  elapsedMs,
  sort,
  onSortChange,
  pageSize,
  onPageSizeChange,
}: Props) {
  return (
    <div className="flex flex-col gap-2 border-b border-paper-rule/60 pb-3 sm:flex-row sm:items-center sm:justify-between">
      <p
        className="font-serif text-xl font-bold tracking-tight text-paper-ink"
        data-testid="result-count"
      >
        {total.toLocaleString()}
        <span className="ml-2 text-sm font-normal italic text-paper-brown">
          results
        </span>
        {typeof elapsedMs === "number" && (
          <span
            className="ml-2 font-mono text-[11px] font-normal text-paper-fade"
            data-testid="elapsed-time"
          >
            · {(elapsedMs / 1000).toFixed(2)}s
          </span>
        )}
      </p>
      <div className="flex flex-wrap items-center gap-3 text-xs">
        <div className="flex items-center gap-1.5">
          <span className="text-paper-brown">Sort</span>
          <Select value={sort} onValueChange={onSortChange}>
            <SelectTrigger className="h-8 w-[140px]">
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
        <div className="flex items-center gap-1.5">
          <span className="text-paper-brown">Per page</span>
          <Select
            value={String(pageSize)}
            onValueChange={(v) => onPageSizeChange(Number(v))}
          >
            <SelectTrigger className="h-8 w-[70px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {[100, 500, 1000, 5000, 10000].map((n) => (
                <SelectItem key={n} value={String(n)}>
                  {n.toLocaleString()}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  );
}
