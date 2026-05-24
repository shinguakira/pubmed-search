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
    <header className="border-b bg-gradient-to-r from-pubmed-dark via-pubmed to-pubmed shadow-sm">
      <div className="container flex h-14 items-center gap-6">
        <Link to="/" className="flex items-center gap-2 text-white">
          <div className="flex h-8 w-8 items-center justify-center rounded-md bg-white/15 ring-1 ring-white/20 backdrop-blur">
            <span className="font-serif text-base font-bold tracking-tight">P</span>
          </div>
          <div className="flex flex-col leading-tight">
            <span className="text-base font-semibold tracking-tight">PubMed</span>
            <span className="text-[10px] uppercase tracking-wider text-white/70">
              shadcn rebuild
            </span>
          </div>
        </Link>
        <nav className="ml-auto flex items-center gap-1 text-sm text-white/90">
          <Button
            variant="ghost"
            size="sm"
            className="text-white hover:bg-white/10 hover:text-white"
            onClick={onOpenSaved}
          >
            <BookmarkIcon className="h-4 w-4" />
            Saved
            {items.length > 0 && (
              <Badge
                variant="secondary"
                className="ml-1 h-5 bg-white/20 px-1.5 text-white"
              >
                {items.length}
              </Badge>
            )}
          </Button>
          <a
            className="rounded-md px-3 py-1.5 text-sm hover:bg-white/10"
            href="https://pubmed.ncbi.nlm.nih.gov/"
            target="_blank"
            rel="noreferrer"
          >
            NCBI ↗
          </a>
        </nav>
      </div>
    </header>
  );
}
