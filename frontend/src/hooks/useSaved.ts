import { useCallback, useEffect, useState } from "react";
import type { Summary } from "@/lib/api";

const KEY = "pubmed.saved.v1";
const EVT = "pubmed-saved-changed";

function read(): Summary[] {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return [];
    return JSON.parse(raw) as Summary[];
  } catch {
    return [];
  }
}

export function useSaved() {
  const [items, setItems] = useState<Summary[]>(() => read());

  useEffect(() => {
    const handler = () => setItems(read());
    window.addEventListener("storage", handler);
    window.addEventListener(EVT, handler);
    return () => {
      window.removeEventListener("storage", handler);
      window.removeEventListener(EVT, handler);
    };
  }, []);

  const persist = (next: Summary[]) => {
    setItems(next);
    localStorage.setItem(KEY, JSON.stringify(next));
    window.dispatchEvent(new Event(EVT));
  };

  const has = useCallback(
    (pmid: string) => items.some((s) => s.pmid === pmid),
    [items],
  );

  const toggle = useCallback(
    (summary: Summary) => {
      const exists = items.some((s) => s.pmid === summary.pmid);
      persist(exists ? items.filter((s) => s.pmid !== summary.pmid) : [...items, summary]);
    },
    [items],
  );

  const remove = useCallback(
    (pmid: string) => persist(items.filter((s) => s.pmid !== pmid)),
    [items],
  );

  const clear = useCallback(() => persist([]), []);

  return { items, has, toggle, remove, clear };
}
