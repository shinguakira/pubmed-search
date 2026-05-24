import { useEffect, useState } from "react";
import { useQuery, keepPreviousData } from "@tanstack/react-query";
import { useSearchParams } from "react-router-dom";

import { Header } from "@/components/Header";
import { SearchBar } from "@/components/SearchBar";
import {
  FiltersSidebar,
  emptyFilters,
  filtersToQueryFragments,
  type Filters,
} from "@/components/FiltersSidebar";
import { ResultsToolbar } from "@/components/ResultsToolbar";
import { ResultItem } from "@/components/ResultItem";
import { Pagination } from "@/components/Pagination";
import { CiteDialog } from "@/components/CiteDialog";
import { SavedDialog } from "@/components/SavedDialog";
import { search } from "@/lib/api";

export default function App() {
  const [searchParams, setSearchParams] = useSearchParams();
  const term = searchParams.get("q") ?? "";
  const page = Number(searchParams.get("page") ?? "1");
  const sort = searchParams.get("sort") ?? "relevance";
  const pageSize = Number(searchParams.get("ps") ?? "20");
  const display = searchParams.get("display") ?? "summary";

  const [filters, setFilters] = useState<Filters>(emptyFilters);
  const [citePmid, setCitePmid] = useState<string | null>(null);
  const [savedOpen, setSavedOpen] = useState(false);

  const setParam = (patch: Record<string, string | number | null>) => {
    const next = new URLSearchParams(searchParams);
    for (const [k, v] of Object.entries(patch)) {
      if (v === null || v === "" || v === undefined) next.delete(k);
      else next.set(k, String(v));
    }
    setSearchParams(next, { replace: true });
  };

  const fragments = filtersToQueryFragments(filters);
  const enabled = term.trim().length > 0;

  const query = useQuery({
    enabled,
    queryKey: ["search", term, page, pageSize, sort, fragments.join("|")],
    queryFn: () =>
      search({
        term,
        page,
        pageSize,
        sort,
        filters: fragments,
      }),
    placeholderData: keepPreviousData,
  });

  useEffect(() => {
    // reset to page 1 when filters change
    if (page !== 1) setParam({ page: 1 });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fragments.join("|"), term, sort, pageSize]);

  return (
    <div className="min-h-screen bg-paper-light text-paper-ink">
      <Header onOpenSaved={() => setSavedOpen(true)} />
      <SearchBar
        value={term}
        onSubmit={(t) => setParam({ q: t, page: 1 })}
      />

      {!enabled ? (
        <EmptyState onPick={(t) => setParam({ q: t, page: 1 })} />
      ) : (
        <main className="w-full px-3 py-4">
          <div className="grid grid-cols-1 gap-3 md:grid-cols-[240px_minmax(0,1fr)] lg:grid-cols-[260px_minmax(0,1fr)]">
            <aside className="border-2 border-paper-rule/70 bg-paper-light p-3 shadow-sm shadow-paper-brown/10">
              <FiltersSidebar value={filters} onChange={setFilters} />
            </aside>

            <section className="min-w-0 border-2 border-paper-rule/70 bg-paper shadow-sm shadow-paper-brown/10">
              <div className="border-b border-paper-rule/60 bg-paper-dark/40 px-6 pt-5">
                <p className="mb-2 text-center font-serif text-[10px] uppercase tracking-[0.4em] text-paper-brown">
                  ── The PubMed Gazette · vol. 1 ──
                </p>
                <ResultsToolbar
                  total={query.data?.count ?? 0}
                  query={query.data?.query_translation ?? term}
                  elapsedMs={query.data?.elapsed_ms}
                  sort={sort}
                  onSortChange={(s) => setParam({ sort: s })}
                  pageSize={pageSize}
                  onPageSizeChange={(n) => setParam({ ps: n, page: 1 })}
                  display={display}
                  onDisplayChange={(d) => setParam({ display: d })}
                />
              </div>

              <div className="px-6 py-2">
                {query.isError && (
                  <div className="my-4 rounded-md border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
                    {(query.error as Error).message}
                  </div>
                )}

                {query.isLoading ? (
                  <ResultsSkeleton />
                ) : query.data?.results.length === 0 ? (
                  <div className="py-16 text-center font-serif italic text-paper-brown">
                    No dispatches found. Try broadening the search or removing filters.
                  </div>
                ) : (
                  <div>
                    {query.data?.results.map((r, i) => (
                      <ResultItem
                        key={r.pmid}
                        index={(page - 1) * pageSize + i + 1}
                        item={r}
                        display={display}
                        onCite={setCitePmid}
                      />
                    ))}
                  </div>
                )}
              </div>

              {query.data && query.data.count > 0 && (
                <div className="border-t border-paper-rule/60 bg-paper-dark/40 px-6">
                  <Pagination
                    page={page}
                    pageSize={pageSize}
                    total={query.data.count}
                    onPageChange={(p) => setParam({ page: p })}
                  />
                </div>
              )}
            </section>
          </div>
        </main>
      )}

      <CiteDialog pmid={citePmid} onOpenChange={(b) => !b && setCitePmid(null)} />
      <SavedDialog open={savedOpen} onOpenChange={setSavedOpen} />
    </div>
  );
}

function ResultsSkeleton() {
  return (
    <div className="space-y-4 py-4">
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="space-y-2">
          <div className="h-4 w-3/4 animate-pulse rounded bg-muted" />
          <div className="h-3 w-1/2 animate-pulse rounded bg-muted/60" />
          <div className="h-3 w-1/3 animate-pulse rounded bg-muted/60" />
        </div>
      ))}
    </div>
  );
}

const SUGGESTED = [
  "CRISPR Cas9",
  "long covid",
  "GLP-1 receptor agonists",
  "Alzheimer disease biomarkers",
  "machine learning radiology",
];

function EmptyState({ onPick }: { onPick: (t: string) => void }) {
  return (
    <main className="w-full px-4 py-16">
      <div className="mx-auto max-w-2xl text-center">
        <p className="font-mono text-[10px] uppercase tracking-[0.4em] text-paper-brown">
          ── No. 1 · Today's Edition ──
        </p>
        <h1 className="mt-3 font-serif text-4xl font-bold tracking-tight text-paper-ink">
          Thirty million letters from the biomedical archive.
        </h1>
        <p className="mt-4 font-serif text-base italic text-paper-sepia">
          Make an inquiry above, or peruse one of today's trending subjects.
        </p>
        <div className="mt-8 flex flex-wrap justify-center gap-2">
          {SUGGESTED.map((s) => (
            <button
              key={s}
              onClick={() => onPick(s)}
              className="border border-paper-rule bg-paper px-3 py-1 font-serif text-sm italic text-paper-sepia transition-colors hover:border-paper-rust hover:text-paper-rust"
            >
              {s}
            </button>
          ))}
        </div>
      </div>
    </main>
  );
}
