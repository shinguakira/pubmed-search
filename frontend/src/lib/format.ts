import type { ArticleDetail } from "./api";

/// Client-side BibTeX / CSV / JSON renderers — used only by the
/// "Individual" export mode to assemble a download from N separate
/// /api/article/{pmid} calls. The "Bulk" mode delegates to the
/// backend's /api/search/export which does the same thing server-side.

export function toBibtex(articles: ArticleDetail[]): string {
  let out = "";
  for (const d of articles) {
    const year = (d.pubdate || "").split(/\s+/)[0] || "";
    const authors = d.authors
      .map((a) => `${a.last_name}, ${a.fore_name}`)
      .join(" and ");
    out +=
      `@article{pmid${d.pmid},\n` +
      `  title   = { ${d.title} },\n` +
      `  author  = { ${authors} },\n` +
      `  journal = { ${d.journal} },\n` +
      `  year    = { ${year} },\n` +
      `  doi     = { ${d.doi} },\n` +
      `  pmid    = { ${d.pmid} }\n` +
      `}\n\n`;
  }
  return out;
}

function csvEscape(s: string): string {
  if (/[",\n]/.test(s)) {
    return `"${s.replace(/"/g, '""')}"`;
  }
  return s;
}

export function toCsv(articles: ArticleDetail[]): string {
  let out = "PMID,Title,Authors,Journal,PubDate,DOI\n";
  for (const d of articles) {
    const authors = d.authors
      .map((a) => `${a.last_name} ${a.fore_name}`.trim())
      .join("; ");
    out +=
      [d.pmid, d.title, authors, d.journal, d.pubdate, d.doi]
        .map(csvEscape)
        .join(",") + "\n";
  }
  return out;
}

export function toJson(articles: ArticleDetail[]): string {
  return JSON.stringify(articles, null, 2);
}

export function triggerDownload(body: string, filename: string, mime: string) {
  const blob = new Blob([body], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

export interface FormatMeta {
  mime: string;
  filename: string;
  render: (articles: ArticleDetail[]) => string;
}

export const FORMATS: Record<"bibtex" | "csv" | "json", FormatMeta> = {
  bibtex: {
    mime: "application/x-bibtex",
    filename: "pubmed.bib",
    render: toBibtex,
  },
  csv: {
    mime: "text/csv",
    filename: "pubmed.csv",
    render: toCsv,
  },
  json: {
    mime: "application/json",
    filename: "pubmed.json",
    render: toJson,
  },
};
