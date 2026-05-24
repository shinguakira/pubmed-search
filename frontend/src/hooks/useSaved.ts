import { useEffect, useState, useCallback } from "react";
import type { Summary } from "@/lib/api";

/// In-memory, tab-scoped store of saved articles. No localStorage, no
/// IndexedDB — bookmarks live only for the current page session. All
/// components that mount `useSaved` share the same underlying Map via
/// a module-level subscription list.

const store = new Map<string, Summary>();
const subscribers = new Set<() => void>();

function emit() {
  for (const fn of subscribers) fn();
}

export function useSaved() {
  // Bump on every change so subscribed components re-render.
  const [, setTick] = useState(0);
  useEffect(() => {
    const fn = () => setTick((t) => t + 1);
    subscribers.add(fn);
    return () => {
      subscribers.delete(fn);
    };
  }, []);

  const items = Array.from(store.values());

  const has = useCallback((pmid: string) => store.has(pmid), []);

  const toggle = useCallback((summary: Summary) => {
    if (store.has(summary.pmid)) store.delete(summary.pmid);
    else store.set(summary.pmid, summary);
    emit();
  }, []);

  const remove = useCallback((pmid: string) => {
    store.delete(pmid);
    emit();
  }, []);

  const clear = useCallback(() => {
    store.clear();
    emit();
  }, []);

  return { items, has, toggle, remove, clear };
}
