import { Link } from "react-router-dom";
import { BookmarkIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSaved } from "@/hooks/useSaved";
import { Badge } from "@/components/ui/badge";

interface HeaderProps {
  onOpenSaved: () => void;
}

export function Header({ onOpenSaved }: HeaderProps) {
  const { items } = useSaved();
  return (
    <header className="border-b-4 border-double border-paper-rust/70 bg-paper-ink text-paper-light">
      <div className="w-full px-4 py-3">
        <div className="flex items-center gap-6">
          <Link to="/" className="flex items-center gap-2">
            <span className="font-serif text-xl font-bold tracking-tight">
              PubMed
            </span>
          </Link>
          <nav className="ml-auto flex items-center gap-3 font-serif text-sm">
            <Button
              variant="ghost"
              size="sm"
              className="border border-paper-light/30 bg-transparent font-serif text-paper-light hover:bg-paper-light/10 hover:text-paper-light"
              onClick={onOpenSaved}
            >
              <BookmarkIcon className="h-4 w-4" />
              Saved
              {items.length > 0 && (
                <Badge
                  variant="secondary"
                  className="ml-1 h-5 bg-paper-rust px-1.5 font-mono text-[10px] text-paper-light"
                >
                  {items.length}
                </Badge>
              )}
            </Button>
            <a
              className="font-serif text-sm italic text-paper-fade hover:text-paper-light hover:underline"
              href="https://pubmed.ncbi.nlm.nih.gov/"
              target="_blank"
              rel="noreferrer"
            >
              NCBI ↗
            </a>
          </nav>
        </div>
      </div>
    </header>
  );
}
