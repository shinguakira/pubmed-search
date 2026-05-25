import { useState, type FormEvent } from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AdvancedBuilder } from "@/components/AdvancedBuilder";

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
}

export function SearchBar({ value, onSubmit }: SearchBarProps) {
  const [term, setTerm] = useState(value);
  const [advancedOpen, setAdvancedOpen] = useState(false);

  const submit = (e?: FormEvent) => {
    e?.preventDefault();
    if (term.trim()) onSubmit(term.trim());
  };

  return (
    <div className="border-b-2 border-paper-rule/70 bg-paper-light">
      <div className="w-full px-4 py-3">
        <form onSubmit={submit} className="flex flex-col gap-1.5">
          <div className="flex w-full items-stretch overflow-hidden border-2 border-paper-ink bg-paper focus-within:border-paper-rust">
            <Input
              value={term}
              onChange={(e) => setTerm(e.target.value)}
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
          <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 text-xs">
            <button
              type="button"
              className="font-serif italic text-paper-rust underline-offset-2 hover:underline"
              onClick={() => setAdvancedOpen(true)}
            >
              › Advanced builder
            </button>
            <div className="flex flex-wrap items-center gap-1.5">
              <span className="font-mono text-[10px] uppercase tracking-[0.14em] text-paper-brown">
                Try
              </span>
              {HINTS.map((h) => (
                <button
                  key={h}
                  type="button"
                  onClick={() => {
                    setTerm(h);
                    onSubmit(h);
                  }}
                  className="border border-paper-rule bg-paper px-2 py-0.5 font-mono text-[10px] uppercase tracking-[0.14em] text-paper-sepia transition-colors hover:border-paper-rust hover:bg-paper-light hover:text-paper-rust"
                >
                  {h}
                </button>
              ))}
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
