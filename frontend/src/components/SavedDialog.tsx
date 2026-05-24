import { Link } from "react-router-dom";
import { Trash2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { useSaved } from "@/hooks/useSaved";

interface Props {
  open: boolean;
  onOpenChange: (b: boolean) => void;
}

export function SavedDialog({ open, onOpenChange }: Props) {
  const { items, remove, clear } = useSaved();

  const exportJson = () => {
    const blob = new Blob([JSON.stringify(items, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "pubmed-saved.json";
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[80vh] max-w-3xl overflow-hidden">
        <DialogHeader>
          <DialogTitle>Saved articles</DialogTitle>
          <DialogDescription>
            Stored locally in your browser. {items.length} saved.
          </DialogDescription>
        </DialogHeader>

        <div className="-mx-6 max-h-[55vh] overflow-y-auto px-6">
          {items.length === 0 ? (
            <div className="py-12 text-center text-sm text-muted-foreground">
              No saved articles yet. Click the bookmark on any result to save.
            </div>
          ) : (
            <ul className="divide-y">
              {items.map((s) => (
                <li key={s.pmid} className="flex gap-3 py-3">
                  <div className="min-w-0 flex-1">
                    <Link
                      to={`/article/${s.pmid}`}
                      onClick={() => onOpenChange(false)}
                      className="text-sm font-medium text-pubmed hover:underline"
                    >
                      {s.title}
                    </Link>
                    <p className="mt-0.5 text-xs text-muted-foreground">
                      {s.authors.slice(0, 4).join(", ")}
                      {s.authors.length > 4 && ", et al."}
                    </p>
                    <p className="text-xs italic text-muted-foreground">
                      {s.source}. {s.pubdate} · PMID {s.pmid}
                    </p>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => remove(s.pmid)}
                    aria-label="Remove"
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {items.length > 0 && (
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="ghost" onClick={clear}>
              Clear all
            </Button>
            <Button onClick={exportJson}>Export JSON</Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
