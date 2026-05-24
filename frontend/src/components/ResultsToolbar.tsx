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
    <div className="flex flex-col gap-2 border-b pb-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0">
        <p className="text-2xl font-semibold tracking-tight text-foreground">
          {total.toLocaleString()}{" "}
          <span className="text-base font-normal text-muted-foreground">
            results
          </span>
        </p>
        {query && (
          <p className="truncate text-xs text-muted-foreground">
            for <code className="rounded bg-muted px-1.5 py-0.5">{query}</code>
          </p>
        )}
      </div>
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">Display</span>
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
          <span className="text-xs text-muted-foreground">Sort by</span>
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
          <span className="text-xs text-muted-foreground">Per page</span>
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
