import { useEffect, useRef, useState } from "react";
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
import { Spinner } from "@/components/ui/spinner";
import { search, type SearchResponse } from "@/lib/api";

export default function App() {
  const [searchParams, setSearchParams] = useSearchParams();
  const term = searchParams.get("q") ?? "";
  const page = Number(searchParams.get("page") ?? "1");
  const sort = searchParams.get("sort") ?? "relevance";
  const pageSize = Number(searchParams.get("ps") ?? "20");
  const display = searchParams.get("display") ?? "summary";
  const bulk = searchParams.get("bulk") === "1";

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

  const [data, setData] = useState<SearchResponse | undefined>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | undefined>();

  // Cancel in-flight requests when params change so a slow earlier call
  // never overwrites a faster later one.
  const inflight = useRef(0);
  const key = `${term}|${page}|${pageSize}|${sort}|${fragments.join(",")}|${bulk}`;

  useEffect(() => {
    if (!enabled) {
      setData(undefined);
      return;
    }
    const myReq = ++inflight.current;
    setLoading(true);
    setError(undefined);
    search({
      term,
      page,
      pageSize,
      sort,
      filters: fragments,
      bulk,
    })
      .then((res) => {
        if (inflight.current === myReq) setData(res);
      })
      .catch((e) => {
        if (inflight.current === myReq) setError(e as Error);
      })
      .finally(() => {
        if (inflight.current === myReq) setLoading(false);
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, enabled]);

  useEffect(() => {
    if (page !== 1) setParam({ page: 1 });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fragments.join("|"), term, sort, pageSize]);

  return (
    <div className="min-h-screen bg-paper-light text-paper-ink">
      <Header onOpenSaved={() => setSavedOpen(true)} />
      <SearchBar
        value={term}
        onSubmit={(t) => setParam({ q: t, page: 1 })}
        bulk={bulk}
        onBulkChange={(b) => setParam({ bulk: b ? "1" : null })}
      />

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
                total={data?.count ?? 0}
                query={data?.query_translation ?? term}
                elapsedMs={data?.elapsed_ms}
                sort={sort}
                onSortChange={(s) => setParam({ sort: s })}
                pageSize={pageSize}
                onPageSizeChange={(n) => setParam({ ps: n, page: 1 })}
                display={display}
                onDisplayChange={(d) => setParam({ display: d })}
              />
            </div>

            <div className="px-6 py-2">
              {!enabled ? (
                <div className="py-16 text-center font-serif italic text-paper-brown">
                  Enter a query in the search bar above to begin.
                </div>
              ) : error ? (
                <div className="my-4 rounded-md border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
                  {error.message}
                </div>
              ) : loading && !data ? (
                <div className="flex justify-center py-16">
                  <Spinner size="lg" label="Inquiring of the archive…" />
                </div>
              ) : data?.results.length === 0 ? (
                <div className="py-16 text-center font-serif italic text-paper-brown">
                  No dispatches found. Try broadening the search or removing
                  filters.
                </div>
              ) : (
                <div className="relative">
                  {loading && data && (
                    <div className="absolute right-0 top-0 z-10 -mt-8">
                      <Spinner size="sm" label="Refreshing…" />
                    </div>
                  )}
                  {data?.results.map((r, i) => (
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

            {data && data.count > 0 && (
              <div className="border-t border-paper-rule/60 bg-paper-dark/40 px-6">
                <Pagination
                  page={page}
                  pageSize={pageSize}
                  total={data.count}
                  onPageChange={(p) => setParam({ page: p })}
                />
              </div>
            )}
          </section>
        </div>
      </main>

      <CiteDialog pmid={citePmid} onOpenChange={(b) => !b && setCitePmid(null)} />
      <SavedDialog open={savedOpen} onOpenChange={setSavedOpen} />
    </div>
  );
}


