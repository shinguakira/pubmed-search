import { useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { ArrowLeft, BookmarkCheck, BookmarkIcon, ExternalLink, Quote } from "lucide-react";

import { getArticle, type ArticleDetail } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { CiteDialog } from "@/components/CiteDialog";
import { SavedDialog } from "@/components/SavedDialog";
import { Header } from "@/components/Header";
import { Spinner } from "@/components/ui/spinner";
import { useSaved } from "@/hooks/useSaved";

export function ArticlePage() {
  const { pmid = "" } = useParams<{ pmid: string }>();
  const [citeOpen, setCiteOpen] = useState(false);
  const [savedOpen, setSavedOpen] = useState(false);
  const { has, toggle } = useSaved();

  const [data, setData] = useState<ArticleDetail | undefined>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | undefined>();

  const inflight = useRef(0);
  useEffect(() => {
    if (!pmid) return;
    const myReq = ++inflight.current;
    setLoading(true);
    setError(undefined);
    setData(undefined);
    getArticle(pmid)
      .then((d) => {
        if (inflight.current === myReq) setData(d);
      })
      .catch((e) => {
        if (inflight.current === myReq) setError(e as Error);
      })
      .finally(() => {
        if (inflight.current === myReq) setLoading(false);
      });
  }, [pmid]);

  const saved = has(pmid);

  return (
    <div className="min-h-screen bg-paper-light text-paper-ink">
      <Header onOpenSaved={() => setSavedOpen(true)} />
      <div className="container py-6">
        <Link to="/" className="inline-flex items-center gap-1 text-sm text-pubmed hover:underline">
          <ArrowLeft className="h-4 w-4" />
          Back to results
        </Link>

        {loading && (
          <div className="mt-16 flex justify-center">
            <Spinner size="lg" label="Fetching article…" />
          </div>
        )}

        {error && (
          <div className="mt-6 rounded-md border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
            {error.message}
          </div>
        )}

        {data && (
          <article className="mt-4 grid grid-cols-1 gap-8 lg:grid-cols-[minmax(0,1fr)_280px]">
            <div className="min-w-0 space-y-6">
              <header className="space-y-2 border-b pb-5">
                <h1
                  className="font-serif text-2xl font-semibold leading-tight tracking-tight"
                  dangerouslySetInnerHTML={{ __html: data.title }}
                />
                {data.journal && (
                  <p className="text-sm italic text-muted-foreground">
                    {data.journal}
                    {data.pubdate && ` · ${data.pubdate}`}
                    {data.doi && ` · doi:${data.doi}`}
                  </p>
                )}
                <p className="text-sm text-foreground/85">
                  {data.authors.map((a) => `${a.fore_name} ${a.last_name}`.trim()).join(", ")}
                </p>
              </header>

              <section className="space-y-3">
                <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                  Abstract
                </h2>
                {data.abstract_text ? (
                  <div className="space-y-4 font-serif text-[15px] leading-relaxed text-foreground/90">
                    {data.abstract_text.split("\n\n").map((p) => (
                      <p key={p} className="whitespace-pre-line">
                        {p}
                      </p>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">No abstract available.</p>
                )}
              </section>

              {data.references.length > 0 && (
                <section className="space-y-2">
                  <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                    References ({data.references.length})
                  </h2>
                  <ol className="list-decimal space-y-1.5 pl-5 text-xs text-foreground/80">
                    {data.references.map((r) => (
                      <li key={r.pmid ?? r.doi ?? r.citation} className="leading-snug">
                        {r.citation || (
                          <span className="italic text-muted-foreground">(no citation text)</span>
                        )}
                        {(r.pmid || r.doi) && (
                          <span className="ml-1 font-mono text-[10px] text-muted-foreground">
                            {r.pmid && (
                              <Link
                                to={`/article/${r.pmid}`}
                                className="text-pubmed hover:underline"
                              >
                                PMID {r.pmid}
                              </Link>
                            )}
                            {r.pmid && r.doi && " · "}
                            {r.doi && (
                              <a
                                href={`https://doi.org/${r.doi}`}
                                target="_blank"
                                rel="noreferrer"
                                className="text-pubmed hover:underline"
                              >
                                DOI {r.doi}
                              </a>
                            )}
                          </span>
                        )}
                      </li>
                    ))}
                  </ol>
                </section>
              )}
            </div>

            <aside className="space-y-3">
              <div className="sticky top-4 space-y-3 rounded-xl border bg-card p-4 shadow-sm">
                <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                  Actions
                </p>
                <Button
                  variant="default"
                  className="w-full justify-start"
                  onClick={() => setCiteOpen(true)}
                >
                  <Quote className="h-4 w-4" />
                  Cite
                </Button>
                <Button
                  variant="outline"
                  className="w-full justify-start"
                  onClick={() => {
                    toggle({
                      pmid: data.pmid,
                      title: data.title,
                      authors: data.authors.map((a) =>
                        `${a.last_name} ${a.fore_name?.[0] ?? ""}`.trim(),
                      ),
                      source: data.journal,
                      pubdate: data.pubdate,
                      epubdate: "",
                      volume: "",
                      issue: "",
                      pages: "",
                      doi: data.doi,
                      pubtypes: data.pubtypes,
                      lang: "",
                    });
                  }}
                >
                  {saved ? (
                    <BookmarkCheck className="h-4 w-4 text-pubmed" />
                  ) : (
                    <BookmarkIcon className="h-4 w-4" />
                  )}
                  {saved ? "Saved" : "Save"}
                </Button>
                <Separator />
                {data.doi && (
                  <a
                    href={`https://doi.org/${data.doi}`}
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                  >
                    <ExternalLink className="h-4 w-4" />
                    Publisher (DOI)
                  </a>
                )}
                <a
                  href={`https://pubmed.ncbi.nlm.nih.gov/${data.pmid}/`}
                  target="_blank"
                  rel="noreferrer"
                  className="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-accent"
                >
                  <ExternalLink className="h-4 w-4" />
                  View on PubMed
                </a>
              </div>
            </aside>
          </article>
        )}
      </div>

      <CiteDialog pmid={citeOpen ? pmid : null} onOpenChange={(b) => setCiteOpen(b)} />
      <SavedDialog open={savedOpen} onOpenChange={setSavedOpen} />
    </div>
  );
}
