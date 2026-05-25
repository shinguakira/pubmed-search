import { useEffect, useRef, useState } from "react";
import { X } from "lucide-react";

import { getArticle, type ArticleDetail } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import {
  CarrierBird,
  FoldedNewsletter,
  LetterOpener,
  RolledLetter,
  SealedEnvelope,
} from "@/components/icons/MailIcons";

export type DrawerAnim = "envelope" | "unfold" | "drop";
export type AnimChoice = "random" | DrawerAnim;

export const DRAWER_ANIMS: DrawerAnim[] = ["envelope", "unfold", "drop"];
export function pickRandomAnim(): DrawerAnim {
  return DRAWER_ANIMS[Math.floor(Math.random() * DRAWER_ANIMS.length)];
}

interface Props {
  pmid: string | null;
  variant: DrawerAnim;
  onClose: () => void;
}

const ICONS: Record<DrawerAnim, (props: { className?: string }) => JSX.Element> = {
  envelope: SealedEnvelope,
  unfold: FoldedNewsletter,
  drop: RolledLetter,
};

export function ArticleDrawer({ pmid, variant, onClose }: Props) {
  const [data, setData] = useState<ArticleDetail | undefined>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | undefined>();
  const inflight = useRef(0);

  useEffect(() => {
    if (!pmid) {
      setData(undefined);
      setError(undefined);
      return;
    }
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

  // ESC to close.
  useEffect(() => {
    if (!pmid) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [pmid, onClose]);

  if (!pmid) return null;
  const Icon = ICONS[variant];

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-paper-ink/45 backdrop-blur-[2px]"
      data-anim={variant}
      onClick={onClose}
      data-testid="article-modal-backdrop"
    >
      {/* Phase 1 — a paper/mail icon flies from off-screen toward centre.
         Envelope variant adds a carrier bird (with flapping wings) that
         delivers the letter and then peels off to the upper-left. */}
      <span className="mail-stage" aria-hidden>
        <span className="mail-courier">
          {variant === "envelope" && (
            <span className="mail-bird">
              <CarrierBird />
            </span>
          )}
          <Icon className="mail-icon" />
        </span>
      </span>

      {/* Envelope-only: a letter opener sweeps across the modal before
          the flap rotates open, as if cutting the envelope along its
          top edge. */}
      {variant === "envelope" && (
        <span className="anim-knife" aria-hidden>
          <LetterOpener />
        </span>
      )}

      {/* Phase 2 — the panel itself materialises and its contents open. */}
      <aside
        key={`${pmid}-${variant}`}
        data-testid="article-drawer"
        className="modal-panel relative max-h-[88vh] w-[92vw] max-w-3xl overflow-y-auto border-2 border-paper-rule/70 bg-paper shadow-2xl shadow-paper-ink/40"
        onClick={(e) => e.stopPropagation()}
      >
        {variant === "envelope" && <span className="anim-flap" aria-hidden />}

        <div className="sticky top-0 z-10 flex items-center justify-between border-b border-paper-rule/60 bg-paper-dark/60 px-4 py-2">
          <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-brown">
            PMID&nbsp;{pmid}
          </span>
          <div className="flex items-center gap-1">
            <a
              href={`https://pubmed.ncbi.nlm.nih.gov/${pmid}/`}
              target="_blank"
              rel="noreferrer"
              className="font-mono text-[10px] uppercase tracking-[0.18em] text-paper-rust hover:underline"
            >
              PubMed ↗
            </a>
            <Button
              variant="ghost"
              size="icon"
              className="h-7 w-7"
              onClick={onClose}
              aria-label="Close article"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div className="px-6 py-5">
          {loading && (
            <div className="flex min-h-[280px] items-center justify-center">
              <Spinner size="lg" label="Fetching article…" />
            </div>
          )}

          {error && (
            <div className="rounded-md border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
              {error.message}
            </div>
          )}

          {data && (
            <article className="space-y-5">
              <header className="anim-section space-y-2 border-b border-paper-rule/60 pb-3">
                <h1
                  className="font-serif text-2xl font-semibold leading-tight tracking-tight text-paper-ink"
                  dangerouslySetInnerHTML={{ __html: data.title }}
                />
                {data.journal && (
                  <p className="text-xs italic text-paper-brown">
                    {data.journal}
                    {data.pubdate && ` · ${data.pubdate}`}
                    {data.doi && (
                      <>
                        {" · "}
                        <a
                          href={`https://doi.org/${data.doi}`}
                          target="_blank"
                          rel="noreferrer"
                          className="text-paper-rust hover:underline"
                        >
                          doi:{data.doi}
                        </a>
                      </>
                    )}
                  </p>
                )}
                <p className="text-xs text-paper-ink/85">
                  {data.authors
                    .map((a) => `${a.fore_name} ${a.last_name}`.trim())
                    .join(", ")}
                </p>
              </header>

              <section className="anim-section space-y-2">
                <h2 className="text-[11px] font-semibold uppercase tracking-wider text-paper-brown">
                  Abstract
                </h2>
                {data.abstract_text ? (
                  <div className="space-y-3 font-serif text-[14px] leading-relaxed text-paper-ink/90">
                    {data.abstract_text.split("\n\n").map((p, i) => (
                      <p key={i} className="whitespace-pre-line">
                        {p}
                      </p>
                    ))}
                  </div>
                ) : (
                  <p className="text-xs italic text-paper-brown">
                    No abstract available.
                  </p>
                )}
              </section>

              {data.references.length > 0 && (
                <section className="anim-section space-y-2">
                  <h2 className="text-[11px] font-semibold uppercase tracking-wider text-paper-brown">
                    References ({data.references.length})
                  </h2>
                  <ol className="list-decimal space-y-1.5 pl-5 text-[11px] text-paper-ink/80">
                    {data.references.map((r, i) => (
                      <li key={i} className="leading-snug">
                        {r.citation || (
                          <span className="italic text-paper-brown">
                            (no citation text)
                          </span>
                        )}
                        {(r.pmid || r.doi) && (
                          <span className="ml-1 font-mono text-[10px] text-paper-brown">
                            {r.pmid && <>PMID {r.pmid}</>}
                            {r.pmid && r.doi && " · "}
                            {r.doi && (
                              <a
                                href={`https://doi.org/${r.doi}`}
                                target="_blank"
                                rel="noreferrer"
                                className="text-paper-rust hover:underline"
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
            </article>
          )}
        </div>
      </aside>
    </div>
  );
}
