import { useState } from "react";
import { Download, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { exportUrl, search as searchApi, getArticle } from "@/lib/api";
import { FORMATS, triggerDownload } from "@/lib/format";
import { cn } from "@/lib/utils";

type Mode = "bulk" | "individual";
type FormatKey = "bibtex" | "csv" | "json";

interface Props {
  term: string;
  sort?: string;
  filters?: string[];
}

interface RunResult {
  mode: Mode;
  format: FormatKey;
  count: number;
  elapsedMs: number;
}

export function ExportButton({ term, sort, filters }: Props) {
  const [mode, setMode] = useState<Mode>("bulk");
  const [format, setFormat] = useState<FormatKey>("bibtex");
  const [count, setCount] = useState<number>(50);
  const [busy, setBusy] = useState(false);
  const [last, setLast] = useState<RunResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const disabled = !term.trim() || busy;

  const run = async () => {
    setBusy(true);
    setError(null);
    const meta = FORMATS[format];
    const started = performance.now();
    try {
      let articleCount = 0;
      if (mode === "bulk") {
        // ONE backend call → backend does esearch(usehistory) + efetch_bulk.
        const url = exportUrl({ term, format, max: count, sort, filters });
        const res = await fetch(url);
        if (!res.ok) throw new Error(`Export failed (${res.status})`);
        const body = await res.text();
        // Crude entry count for reporting.
        articleCount =
          format === "bibtex"
            ? (body.match(/@article\{pmid/g) || []).length
            : format === "csv"
              ? Math.max(0, body.split("\n").filter(Boolean).length - 1)
              : (JSON.parse(body) as unknown[]).length;
        triggerDownload(body, meta.filename, meta.mime);
      } else {
        // INDIVIDUAL: search to get PMIDs, then N x /api/article/{pmid}.
        // Sequential to stay polite to backend / NCBI.
        const s = await searchApi({
          term,
          pageSize: count,
          page: 1,
          sort,
          filters,
        });
        const articles = [];
        for (const r of s.results) {
          const a = await getArticle(r.pmid);
          articles.push(a);
        }
        articleCount = articles.length;
        const body = meta.render(articles);
        triggerDownload(body, meta.filename, meta.mime);
      }
      const elapsedMs = Math.round(performance.now() - started);
      setLast({ mode, format, count: articleCount, elapsedMs });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className="h-9 border-paper-ink bg-paper text-paper-ink hover:bg-paper-dark"
          disabled={!term.trim()}
        >
          <Download className="h-4 w-4" />
          Export
        </Button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        className="w-80 border-2 border-paper-rule bg-paper-light text-paper-ink"
      >
        <div className="space-y-3 font-serif">
          <p className="font-mono text-[10px] uppercase tracking-[0.2em] text-paper-brown">
            Export search results
          </p>

          <div className="space-y-1">
            <Label className="font-mono text-[10px] uppercase tracking-[0.16em] text-paper-brown">
              Format
            </Label>
            <Select
              value={format}
              onValueChange={(v) => setFormat(v as FormatKey)}
            >
              <SelectTrigger className="h-9 border-paper-rule bg-paper">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="bibtex">BibTeX</SelectItem>
                <SelectItem value="csv">CSV</SelectItem>
                <SelectItem value="json">JSON</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-1">
            <Label className="font-mono text-[10px] uppercase tracking-[0.16em] text-paper-brown">
              Mode
            </Label>
            <div className="grid grid-cols-2 overflow-hidden rounded border border-paper-rule">
              <ModeChip
                active={mode === "bulk"}
                onClick={() => setMode("bulk")}
                label="Bulk"
                sub="1 backend call"
              />
              <ModeChip
                active={mode === "individual"}
                onClick={() => setMode("individual")}
                label="Individual"
                sub="N article calls"
              />
            </div>
          </div>

          <div className="space-y-1">
            <Label className="font-mono text-[10px] uppercase tracking-[0.16em] text-paper-brown">
              Count
            </Label>
            <Input
              type="number"
              min={1}
              max={10000}
              value={count}
              onChange={(e) => setCount(Math.max(1, Number(e.target.value) || 1))}
              className="h-9 border-paper-rule bg-paper"
            />
            <p className="font-mono text-[9px] uppercase tracking-[0.1em] text-paper-fade">
              {mode === "individual"
                ? `≈ ${(count * 0.6).toFixed(0)} s expected`
                : `≈ ${Math.max(1, Math.ceil(count / 50))} – ${Math.max(2, Math.ceil(count / 25))} s expected`}
            </p>
          </div>

          <Button
            onClick={run}
            disabled={disabled}
            className="w-full bg-paper-ink font-serif text-sm uppercase tracking-[0.2em] text-paper-light hover:bg-paper-rust"
          >
            {busy ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Running…
              </>
            ) : (
              <>Download</>
            )}
          </Button>

          {last && !busy && (
            <div className="rounded border border-paper-rule bg-paper px-3 py-2 font-mono text-[11px] text-paper-ink">
              <span className="font-semibold uppercase tracking-[0.12em] text-paper-brown">
                Last:
              </span>{" "}
              {last.mode} · {last.format} · {last.count} rec ·{" "}
              <span className="font-semibold text-paper-rust">
                {(last.elapsedMs / 1000).toFixed(2)}s
              </span>
            </div>
          )}
          {error && (
            <div className="rounded border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
              {error}
            </div>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}

function ModeChip({
  active,
  onClick,
  label,
  sub,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
  sub: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "px-2 py-1.5 text-left transition-colors",
        active
          ? "bg-paper-ink text-paper-light"
          : "bg-paper text-paper-ink hover:bg-paper-dark",
      )}
    >
      <div className="font-serif text-[13px] font-semibold leading-tight">
        {label}
      </div>
      <div
        className={cn(
          "font-mono text-[9px] uppercase tracking-[0.1em]",
          active ? "text-paper-light/70" : "text-paper-fade",
        )}
      >
        {sub}
      </div>
    </button>
  );
}
