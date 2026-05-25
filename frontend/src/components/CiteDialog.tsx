import { useEffect, useRef, useState } from "react";
import { Copy, Check } from "lucide-react";
import { getCite, type CiteResponse } from "@/lib/api";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";

interface Props {
  pmid: string | null;
  onOpenChange: (b: boolean) => void;
}

const FORMATS: { id: keyof CiteResponse; label: string }[] = [
  { id: "ama", label: "AMA" },
  { id: "apa", label: "APA" },
  { id: "mla", label: "MLA" },
  { id: "nlm", label: "NLM" },
  { id: "bibtex", label: "BibTeX" },
];

export function CiteDialog({ pmid, onOpenChange }: Props) {
  const [data, setData] = useState<CiteResponse | undefined>();
  const [loading, setLoading] = useState(false);
  const inflight = useRef(0);

  useEffect(() => {
    if (!pmid) {
      setData(undefined);
      return;
    }
    const myReq = ++inflight.current;
    setLoading(true);
    setData(undefined);
    getCite(pmid)
      .then((d) => {
        if (inflight.current === myReq) setData(d);
      })
      .finally(() => {
        if (inflight.current === myReq) setLoading(false);
      });
  }, [pmid]);

  const [copied, setCopied] = useState<string | null>(null);

  const copy = (key: string, text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(key);
    setTimeout(() => setCopied(null), 1500);
  };

  return (
    <Dialog open={Boolean(pmid)} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Cite this article</DialogTitle>
          <DialogDescription>
            Choose a citation style. Copy and paste into your manuscript or
            reference manager.
          </DialogDescription>
        </DialogHeader>

        {loading || !data ? (
          <div className="flex h-40 items-center justify-center">
            <Spinner size="md" label="Composing citation…" />
          </div>
        ) : (
          <Tabs defaultValue="ama" className="w-full">
            <TabsList>
              {FORMATS.map((f) => (
                <TabsTrigger key={f.id} value={f.id}>
                  {f.label}
                </TabsTrigger>
              ))}
            </TabsList>
            {FORMATS.map((f) => (
              <TabsContent key={f.id} value={f.id}>
                <div className="relative">
                  <pre className="max-h-72 overflow-auto whitespace-pre-wrap break-words rounded-md border bg-muted/40 p-4 font-mono text-sm">
                    {data[f.id]}
                  </pre>
                  <Button
                    variant="outline"
                    size="sm"
                    className="absolute right-2 top-2 h-7 px-2 text-xs"
                    onClick={() => copy(f.id, data[f.id])}
                  >
                    {copied === f.id ? (
                      <>
                        <Check className="h-3.5 w-3.5" /> Copied
                      </>
                    ) : (
                      <>
                        <Copy className="h-3.5 w-3.5" /> Copy
                      </>
                    )}
                  </Button>
                </div>
              </TabsContent>
            ))}
          </Tabs>
        )}
      </DialogContent>
    </Dialog>
  );
}
