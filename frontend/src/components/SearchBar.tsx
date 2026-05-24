import { useState, type FormEvent } from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AdvancedBuilder } from "@/components/AdvancedBuilder";

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
    <div className="border-b bg-white/60 backdrop-blur supports-[backdrop-filter]:bg-white/60">
      <div className="container py-4">
        <form onSubmit={submit} className="flex flex-col gap-2">
          <div className="flex w-full items-stretch overflow-hidden rounded-lg border border-input shadow-sm ring-offset-background focus-within:ring-2 focus-within:ring-ring">
            <Input
              value={term}
              onChange={(e) => setTerm(e.target.value)}
              placeholder="Search PubMed..."
              className="h-11 flex-1 rounded-none border-0 px-4 text-base shadow-none focus-visible:ring-0"
              autoFocus
            />
            <Button
              type="submit"
              variant="default"
              className="h-11 rounded-none rounded-r-lg px-5 text-sm font-semibold"
            >
              <Search className="h-4 w-4" />
              Search
            </Button>
          </div>
          <div className="flex items-center justify-between text-xs">
            <button
              type="button"
              className="text-pubmed underline-offset-2 hover:underline"
              onClick={() => setAdvancedOpen(true)}
            >
              Advanced
            </button>
            <p className="text-muted-foreground">
              Try: <code className="rounded bg-muted px-1.5 py-0.5">crispr cas9</code>{" "}
              <code className="rounded bg-muted px-1.5 py-0.5">covid 2024[dp]</code>
            </p>
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
