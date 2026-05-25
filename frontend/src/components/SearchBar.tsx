import { useEffect, useRef, useState, type FormEvent } from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AdvancedBuilder } from "@/components/AdvancedBuilder";
import { cn } from "@/lib/utils";

const HINTS = [
  "crispr cas9",
  "glaucoma OCT",
  "long covid",
  "alzheimer review",
  "GLP-1",
  "diabetes",
];

interface SearchBarProps {
  value: string;
  onSubmit: (term: string) => void;
  bulk: boolean;
  onBulkChange: (b: boolean) => void;
}

export function SearchBar({ value, onSubmit, bulk, onBulkChange }: SearchBarProps) {
  const [term, setTerm] = useState(value);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [focused, setFocused] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);

  // Close the hint dropdown on outside click.
  useEffect(() => {
    if (!focused) return;
    const onDown = (e: MouseEvent) => {
      if (wrapRef.current && !wrapRef.current.contains(e.target as Node)) {
        setFocused(false);
      }
    };
    document.addEventListener("mousedown", onDown);
    return () => document.removeEventListener("mousedown", onDown);
  }, [focused]);

  const submit = (e?: FormEvent) => {
    e?.preventDefault();
    if (term.trim()) {
      onSubmit(term.trim());
      setFocused(false);
    }
  };

  const pickHint = (h: string) => {
    // Only fill the input — do NOT trigger a search. User still has
    // to press the Search button (or Enter) to fetch.
    setTerm(h);
    setFocused(false);
  };

  return (
    <div className="border-b-2 border-paper-rule/70 bg-paper-light">
      <div className="w-full px-4 py-3">
        <form onSubmit={submit} className="space-y-2">
          <div ref={wrapRef} className="relative">
            <div className="flex w-full items-stretch overflow-hidden border-2 border-paper-ink bg-paper focus-within:border-paper-rust">
              <Input
                value={term}
                onChange={(e) => setTerm(e.target.value)}
                onFocus={() => setFocused(true)}
                placeholder="Inquire of the archive…"
                className="h-11 flex-1 rounded-none border-0 bg-transparent px-4 font-serif text-base text-paper-ink placeholder:font-serif placeholder:italic placeholder:text-paper-fade focus-visible:ring-0"
                autoFocus
              />
              <Button
                type="submit"
                variant="default"
                className="h-11 rounded-none border-l-2 border-paper-ink bg-paper-ink px-6 font-serif text-sm font-semibold uppercase tracking-[0.2em] text-paper-light hover:bg-paper-rust"
              >
                <Search className="h-4 w-4" />
                Search
              </Button>
            </div>

            {/* Hint dropdown — only when input is focused. */}
            {focused && (
              <div className="absolute left-0 right-0 top-full z-30 mt-1 border-2 border-paper-rule bg-paper-light shadow-lg shadow-paper-brown/20">
                <p className="border-b border-paper-rule/60 px-3 py-1.5 font-mono text-[10px] uppercase tracking-[0.2em] text-paper-brown">
                  Suggested queries
                </p>
                <ul className="max-h-72 overflow-auto py-1">
                  {HINTS.map((h) => (
                    <li key={h}>
                      <button
                        type="button"
                        onMouseDown={(e) => {
                          // Prevent input blur before click registers.
                          e.preventDefault();
                          pickHint(h);
                        }}
                        className="block w-full px-3 py-1.5 text-left font-serif text-[14px] text-paper-ink hover:bg-paper-dark/60 hover:text-paper-rust"
                      >
                        {h}
                      </button>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>

          {/* Prominent FETCH MODE toggle directly under the search input. */}
          <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1.5">
            <button
              type="button"
              className="font-serif text-xs italic text-paper-rust underline-offset-2 hover:underline"
              onClick={() => setAdvancedOpen(true)}
            >
              › Advanced builder
            </button>
            <div className="flex items-center gap-2">
              <span className="font-mono text-[10px] font-bold uppercase tracking-[0.22em] text-paper-brown">
                Fetch mode
              </span>
              <div className="flex overflow-hidden rounded border-2 border-paper-ink shadow-sm shadow-paper-brown/20">
                <button
                  type="button"
                  onClick={() => onBulkChange(false)}
                  title="esearch + esummary — light, no abstract"
                  className={cn(
                    "px-3 py-1 font-mono text-[11px] font-bold uppercase tracking-[0.16em] transition-colors",
                    !bulk
                      ? "bg-paper-ink text-paper-light"
                      : "bg-paper text-paper-brown hover:bg-paper-dark",
                  )}
                >
                  Default
                </button>
                <button
                  type="button"
                  onClick={() => onBulkChange(true)}
                  title="esearch(usehistory) + efetch_bulk — heavier but populates server cache; subsequent article clicks ~0 ms"
                  className={cn(
                    "px-3 py-1 font-mono text-[11px] font-bold uppercase tracking-[0.16em] transition-colors",
                    bulk
                      ? "bg-paper-rust text-paper-light"
                      : "bg-paper text-paper-brown hover:bg-paper-dark",
                  )}
                >
                  Bulk
                </button>
              </div>
            </div>
          </div>
        </form>

        <AdvancedBuilder
          open={advancedOpen}
          onOpenChange={setAdvancedOpen}
          onApply={(t) => {
            setTerm(t);
            onSubmit(t);
          }}
        />
      </div>
    </div>
  );
}
