import { useMemo, useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface Row {
  id: string;
  op: "AND" | "OR" | "NOT";
  field: string;
  value: string;
}

const FIELDS: { value: string; label: string }[] = [
  { value: "All Fields", label: "All Fields" },
  { value: "Title", label: "Title" },
  { value: "Title/Abstract", label: "Title/Abstract" },
  { value: "Author", label: "Author" },
  { value: "Affiliation", label: "Affiliation" },
  { value: "Journal", label: "Journal" },
  { value: "MeSH Terms", label: "MeSH Terms" },
  { value: "Publication Type", label: "Publication Type" },
  { value: "Date - Publication", label: "Date - Publication" },
];

const FIELD_TAG: Record<string, string> = {
  "All Fields": "",
  Title: "[ti]",
  "Title/Abstract": "[tiab]",
  Author: "[au]",
  Affiliation: "[ad]",
  Journal: "[journal]",
  "MeSH Terms": "[mesh]",
  "Publication Type": "[pt]",
  "Date - Publication": "[dp]",
};

const rid = () => Math.random().toString(36).slice(2, 9);

interface Props {
  open: boolean;
  onOpenChange: (b: boolean) => void;
  onApply: (term: string) => void;
}

export function AdvancedBuilder({ open, onOpenChange, onApply }: Props) {
  const [rows, setRows] = useState<Row[]>([
    { id: rid(), op: "AND", field: "All Fields", value: "" },
    { id: rid(), op: "AND", field: "All Fields", value: "" },
  ]);

  const preview = useMemo(() => buildQuery(rows), [rows]);

  const update = (id: string, patch: Partial<Row>) =>
    setRows((r) => r.map((row) => (row.id === id ? { ...row, ...patch } : row)));
  const remove = (id: string) => setRows((r) => r.filter((row) => row.id !== id));
  const add = () =>
    setRows((r) => [...r, { id: rid(), op: "AND", field: "All Fields", value: "" }]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>Advanced search builder</DialogTitle>
          <DialogDescription>
            Combine terms with boolean operators and field tags. PubMed-compatible
            syntax.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-2">
          {rows.map((row, idx) => (
            <div
              key={row.id}
              className="grid grid-cols-[80px_180px_1fr_auto] items-center gap-2"
            >
              {idx === 0 ? (
                <div className="text-xs font-medium text-muted-foreground">—</div>
              ) : (
                <Select
                  value={row.op}
                  onValueChange={(v) => update(row.id, { op: v as Row["op"] })}
                >
                  <SelectTrigger className="h-9">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="AND">AND</SelectItem>
                    <SelectItem value="OR">OR</SelectItem>
                    <SelectItem value="NOT">NOT</SelectItem>
                  </SelectContent>
                </Select>
              )}
              <Select
                value={row.field}
                onValueChange={(v) => update(row.id, { field: v })}
              >
                <SelectTrigger className="h-9">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {FIELDS.map((f) => (
                    <SelectItem key={f.value} value={f.value}>
                      {f.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Input
                value={row.value}
                onChange={(e) => update(row.id, { value: e.target.value })}
                placeholder="Enter a term"
              />
              <Button
                type="button"
                variant="ghost"
                size="icon"
                onClick={() => remove(row.id)}
                disabled={rows.length <= 1}
                aria-label="Remove row"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          ))}
          <Button type="button" variant="outline" size="sm" onClick={add}>
            <Plus className="h-4 w-4" />
            Add row
          </Button>
        </div>

        <div className="space-y-1.5 rounded-md border bg-muted/40 p-3">
          <p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Query preview
          </p>
          <pre className="whitespace-pre-wrap break-words font-mono text-sm">
            {preview || <span className="text-muted-foreground">(empty)</span>}
          </pre>
        </div>

        <div className="flex justify-end gap-2">
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            disabled={!preview}
            onClick={() => {
              onApply(preview);
              onOpenChange(false);
            }}
          >
            Search
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function buildQuery(rows: Row[]): string {
  const parts: string[] = [];
  for (const r of rows) {
    const v = r.value.trim();
    if (!v) continue;
    const tag = FIELD_TAG[r.field] ?? "";
    const fragment = tag ? `(${v})${tag}` : v;
    if (parts.length === 0) {
      parts.push(fragment);
    } else {
      parts.push(`${r.op} ${fragment}`);
    }
  }
  return parts.join(" ");
}
