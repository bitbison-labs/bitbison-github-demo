// The corral API, as the web UI uses it. In development vite proxies /api
// to the API on 127.0.0.1:8080 (vite.config.ts).

export interface Link {
  slug: string;
  url: string;
  hits: number;
}

const base = import.meta.env.VITE_CORRAL_API ?? "/api";

async function answer<T>(response: Response): Promise<T> {
  const body = await response.json();
  if (!response.ok) {
    throw new Error(body.error ?? `the API answered ${response.status}`);
  }
  return body as T;
}

export async function links(): Promise<Link[]> {
  return answer(await fetch(`${base}/links`));
}

export async function shorten(url: string): Promise<Link> {
  return answer(
    await fetch(`${base}/links`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ url }),
    }),
  );
}

export function shortAddress(link: Link): string {
  return `${base}/${link.slug}`;
}
