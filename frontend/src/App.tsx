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
import {
  ArticleDrawer,
  pickRandomAnim,
  type AnimChoice,
  type DrawerAnim,
} from "@/components/ArticleDrawer";
import { Pagination } from "@/components/Pagination";
import { CiteDialog } from "@/components/CiteDialog";
import { SavedDialog } from "@/components/SavedDialog";
import { Spinner } from "@/components/ui/spinner";
import { search, type SearchResponse } from "@/lib/api";

export default function App() {
  // URL params hold the *applied* search state. The only writers are:
  //   * the Search button (commits everything at once)
  //   * Pagination (page-only writes for navigation)
  // Filter sidebar / sort / pageSize / bulk controls update local
  // "pending" state and do NOT trigger a fetch until the user clicks
  // Search. This matches PubMed's own apply-then-search UX.
  const [searchParams, setSearchParams] = useSearchParams();
  const term = searchParams.get("q") ?? "";
  const page = Number(searchParams.get("page") ?? "1");
  const appliedSort = searchParams.get("sort") ?? "relevance";
  const appliedPageSize = Number(searchParams.get("ps") ?? "100");
  const appliedBulk = searchParams.get("bulk") === "1";
  const appliedFiltersStr = searchParams.get("filters") ?? "";

  // Pending state — bound to the sidebar and toolbar controls.
  const [pendingFilters, setPendingFilters] = useState<Filters>(emptyFilters);
  const [pendingSort, setPendingSort] = useState(appliedSort);
  const [pendingPageSize, setPendingPageSize] = useState(appliedPageSize);
  const [pendingBulk, setPendingBulk] = useState(appliedBulk);

  // When URL changes externally (back/forward, hint chip click, etc.),
  // pull the applied values back into pending so controls reflect what
  // is actually shown.
  useEffect(() => {
    setPendingSort(appliedSort);
  }, [appliedSort]);
  useEffect(() => {
    setPendingPageSize(appliedPageSize);
  }, [appliedPageSize]);
  useEffect(() => {
    setPendingBulk(appliedBulk);
  }, [appliedBulk]);

  const [citePmid, setCitePmid] = useState<string | null>(null);
  const [savedOpen, setSavedOpen] = useState(false);
  const [selectedPmid, setSelectedPmid] = useState<string | null>(null);
  const [anim, setAnim] = useState<AnimChoice>("random");
  const [resolvedAnim, setResolvedAnim] = useState<DrawerAnim>(() =>
    pickRandomAnim(),
  );

  const handleSelectArticle = (pmid: string) => {
    setResolvedAnim(anim === "random" ? pickRandomAnim() : anim);
    setSelectedPmid(pmid);
  };

  const handleAnimChange = (a: AnimChoice) => {
    setAnim(a);
    setResolvedAnim(a === "random" ? pickRandomAnim() : a);
  };

  const setParam = (patch: Record<string, string | number | null>) => {
    const next = new URLSearchParams(searchParams);
    for (const [k, v] of Object.entries(patch)) {
      if (v === null || v === "" || v === undefined) next.delete(k);
      else next.set(k, String(v));
    }
    setSearchParams(next, { replace: true });
  };

  const appliedFragments = appliedFiltersStr
    ? appliedFiltersStr.split(",").filter(Boolean)
    : [];

  // The single fetch effect. Reacts ONLY to URL changes — never to
  // pending state. Filter checkboxes / sort dropdown / bulk toggle do
  // not appear in this dependency list on purpose.
  const [data, setData] = useState<SearchResponse | undefined>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | undefined>();
  const inflight = useRef(0);
  const fetchKey = `${term}|${page}|${appliedPageSize}|${appliedSort}|${appliedFiltersStr}|${appliedBulk}`;
  useEffect(() => {
    setSelectedPmid(null);
    if (!term.trim()) {
      setData(undefined);
      return;
    }
    const myReq = ++inflight.current;
    setLoading(true);
    setError(undefined);
    search({
      term,
      page,
      pageSize: appliedPageSize,
      sort: appliedSort,
      filters: appliedFragments,
      bulk: appliedBulk,
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
  }, [fetchKey]);

  // Search button (and Enter in the search bar) routes through here.
  const applySearch = (newTerm: string) => {
    const fragments = filtersToQueryFragments(pendingFilters);
    setParam({
      q: newTerm,
      page: 1,
      sort: pendingSort === "relevance" ? null : pendingSort,
      ps: pendingPageSize === 100 ? null : pendingPageSize,
      bulk: pendingBulk ? "1" : null,
      filters: fragments.length > 0 ? fragments.join(",") : null,
    });
  };

  const enabled = term.trim().length > 0;

  return (
    <div className="min-h-screen bg-paper-light text-paper-ink">
      <Header
        onOpenSaved={() => setSavedOpen(true)}
        anim={anim}
        onAnimChange={handleAnimChange}
      />
      <SearchBar
        value={term}
        onSubmit={applySearch}
        bulk={pendingBulk}
        onBulkChange={setPendingBulk}
      />

      <main className="w-full px-3 py-4">
        <div className="grid grid-cols-1 gap-3 md:grid-cols-[240px_minmax(0,1fr)] lg:grid-cols-[260px_minmax(0,1fr)]">
          <aside className="border-2 border-paper-rule/70 bg-paper-light p-3 shadow-sm shadow-paper-brown/10">
            <FiltersSidebar value={pendingFilters} onChange={setPendingFilters} />
          </aside>

          <section className="min-w-0 border-2 border-paper-rule/70 bg-paper shadow-sm shadow-paper-brown/10">
            <div className="border-b border-paper-rule/60 bg-paper-dark/40 px-6 pt-4">
              <ResultsToolbar
                total={data?.count ?? 0}
                elapsedMs={data?.elapsed_ms}
                sort={pendingSort}
                onSortChange={setPendingSort}
                pageSize={pendingPageSize}
                onPageSizeChange={setPendingPageSize}
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
              ) : loading ? (
                <div className="flex min-h-[480px] items-center justify-center">
                  <Spinner
                    size="lg"
                    label={
                      data ? "Refreshing dispatches…" : "Inquiring of the archive…"
                    }
                  />
                </div>
              ) : data?.results.length === 0 ? (
                <div className="py-16 text-center font-serif italic text-paper-brown">
                  No dispatches found. Try broadening the search or removing
                  filters.
                </div>
              ) : (
                <div>
                  {data?.results.map((r, i) => (
                    <ResultItem
                      key={r.pmid}
                      index={(page - 1) * appliedPageSize + i + 1}
                      item={r}
                      selected={selectedPmid === r.pmid}
                      onSelect={handleSelectArticle}
                    />
                  ))}
                </div>
              )}
            </div>

            {data && data.count > 0 && (
              <div className="border-t border-paper-rule/60 bg-paper-dark/40 px-6">
                <Pagination
                  page={page}
                  pageSize={appliedPageSize}
                  total={data.count}
                  onPageChange={(p) => setParam({ page: p })}
                />
              </div>
            )}
          </section>
        </div>
      </main>

      {selectedPmid && (
        <ArticleDrawer
          pmid={selectedPmid}
          variant={resolvedAnim}
          onClose={() => setSelectedPmid(null)}
        />
      )}

      <CiteDialog pmid={citePmid} onOpenChange={(b) => !b && setCitePmid(null)} />
      <SavedDialog open={savedOpen} onOpenChange={setSavedOpen} />
    </div>
  );
}
