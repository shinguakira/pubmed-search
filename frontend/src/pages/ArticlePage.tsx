import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  ArrowLeft,
  BookmarkCheck,
  BookmarkIcon,
  ExternalLink,
  Quote,
} from "lucide-react";

import { getArticle } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { CiteDialog } from "@/components/CiteDialog";
import { SavedDialog } from "@/components/SavedDialog";
import { Header } from "@/components/Header";
import { useSaved } from "@/hooks/useSaved";

export function ArticlePage() {
  const { pmid = "" } = useParams<{ pmid: string }>();
  const [citeOpen, setCiteOpen] = useState(false);
  const [savedOpen, setSavedOpen] = useState(false);
  const { has, toggle } = useSaved();

  const { data, isLoading, isError, error } = useQuery({
    queryKey: ["article", pmid],
    queryFn: () => getArticle(pmid),
    enabled: Boolean(pmid),
  });

  const saved = has(pmid);

  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-50/60 via-white to-white">
      <Header onOpenSaved={() => setSavedOpen(true)} />
      <div className="container py-6">
        <Link
          to="/"
          className="inline-flex items-center gap-1 text-sm text-pubmed hover:underline"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to results
        </Link>

        {isLoading && (
          <div className="mt-6 space-y-3">
            <div className="h-8 w-3/4 animate-pulse rounded bg-muted" />
            <div className="h-4 w-1/2 animate-pulse rounded bg-muted/60" />
            <div className="mt-6 h-64 animate-pulse rounded bg-muted/40" />
          </div>
        )}

        {isError && (
          <div className="mt-6 rounded-md border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive">
            {(error as Error).message}
          </div>
        )}

        {data && (
          <article className="mt-4 grid grid-cols-1 gap-8 lg:grid-cols-[minmax(0,1fr)_280px]">
            <div className="min-w-0 space-y-6">
              <header className="space-y-3 border-b pb-6">
                <div className="flex flex-wrap items-center gap-1.5">
                  <Badge variant="outline" className="font-mono text-[10px]">
                    PMID: {data.pmid}
                  </Badge>
                  {data.pubtypes.slice(0, 3).map((t) => (
                    <Badge key={t} variant="soft" className="text-[10px]">
                      {t}
                    </Badge>
                  ))}
                </div>
                <h1
                  className="font-serif text-3xl font-semibold leading-tight tracking-tight"
                  dangerouslySetInnerHTML={{ __html: data.title }}
                />
                {data.journal && (
                  <p className="text-sm italic text-muted-foreground">
                    {data.journal}
                    {data.pubdate && ` · ${data.pubdate}`}
                  </p>
                )}
                <p className="text-sm text-foreground/85">
                  {data.authors
                    .map((a) => `${a.fore_name} ${a.last_name}`.trim())
                    .join(", ")}
                </p>
              </header>

              <section className="space-y-3">
                <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                  Abstract
                </h2>
                {data.abstract_text ? (
                  <div className="space-y-4 font-serif text-[15px] leading-relaxed text-foreground/90">
                    {data.abstract_text.split("\n\n").map((p, i) => (
                      <p key={i} className="whitespace-pre-line">
                        {p}
                      </p>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    No abstract available.
                  </p>
                )}
              </section>

              {data.keywords.length > 0 && (
                <section className="space-y-2">
                  <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                    Keywords
                  </h2>
                  <div className="flex flex-wrap gap-1.5">
                    {data.keywords.map((k) => (
                      <Badge key={k} variant="secondary" className="text-[11px]">
                        {k}
                      </Badge>
                    ))}
                  </div>
                </section>
              )}

              {data.mesh_terms.length > 0 && (
                <section className="space-y-2">
                  <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                    MeSH terms
                  </h2>
                  <div className="flex flex-wrap gap-1.5">
                    {data.mesh_terms.map((k) => (
                      <Badge key={k} variant="outline" className="text-[11px]">
                        {k}
                      </Badge>
                    ))}
                  </div>
                </section>
              )}

              {data.authors.some((a) => a.affiliation) && (
                <section className="space-y-2">
                  <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
                    Affiliations
                  </h2>
                  <ol className="list-decimal space-y-1 pl-5 text-xs text-muted-foreground">
                    {data.authors
                      .filter((a) => a.affiliation)
                      .map((a, i) => (
                        <li key={i}>
                          <span className="font-medium text-foreground/90">
                            {a.fore_name} {a.last_name}
                          </span>{" "}
                          — {a.affiliation}
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
                      authors: data.authors.map(
                        (a) => `${a.last_name} ${a.fore_name?.[0] ?? ""}`.trim(),
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
