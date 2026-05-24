export interface Summary {
  pmid: string;
  title: string;
  authors: string[];
  source: string;
  pubdate: string;
  epubdate: string;
  volume: string;
  issue: string;
  pages: string;
  doi: string;
  pubtypes: string[];
  lang: string;
}

export interface SearchResponse {
  count: number;
  page: number;
  page_size: number;
  query_translation: string;
  results: Summary[];
}

export interface Author {
  last_name: string;
  fore_name: string;
  affiliation: string;
}

export interface ArticleDetail {
  pmid: string;
  title: string;
  abstract_text: string;
  authors: Author[];
  journal: string;
  pubdate: string;
  doi: string;
  keywords: string[];
  mesh_terms: string[];
  pubtypes: string[];
}

export interface MeshTerm {
  id: string;
  name: string;
}

export interface CiteResponse {
  ama: string;
  apa: string;
  mla: string;
  nlm: string;
  bibtex: string;
}

export interface SearchParams {
  term: string;
  page?: number;
  pageSize?: number;
  sort?: string;
  filters?: string[];
}

const BASE = (import.meta.env.VITE_API_URL ?? "http://127.0.0.1:8787") + "/api";

async function getJson<T>(url: string): Promise<T> {
  const res = await fetch(url);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`API ${res.status}: ${text}`);
  }
  return res.json() as Promise<T>;
}

export function search(params: SearchParams): Promise<SearchResponse> {
  const qp = new URLSearchParams();
  qp.set("term", params.term);
  if (params.page) qp.set("page", String(params.page));
  if (params.pageSize) qp.set("page_size", String(params.pageSize));
  if (params.sort) qp.set("sort", params.sort);
  if (params.filters && params.filters.length > 0)
    qp.set("filters", params.filters.join(","));
  return getJson(`${BASE}/search?${qp.toString()}`);
}

export function getArticle(pmid: string): Promise<ArticleDetail> {
  return getJson(`${BASE}/article/${encodeURIComponent(pmid)}`);
}

export function getMesh(term: string, limit = 10) {
  const qp = new URLSearchParams({ term, limit: String(limit) });
  return getJson<{ count: number; terms: MeshTerm[] }>(
    `${BASE}/mesh?${qp.toString()}`,
  );
}

export function getCite(pmid: string): Promise<CiteResponse> {
  return getJson(`${BASE}/cite/${encodeURIComponent(pmid)}`);
}
