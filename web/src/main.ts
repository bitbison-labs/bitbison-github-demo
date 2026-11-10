import { links, shorten, shortAddress, type Link } from "./api";
import "../vendor/normalize.css/normalize.css";
import "./style.css";

const form = document.querySelector<HTMLFormElement>("#new-link")!;
const rows = document.querySelector<HTMLTableSectionElement>("#links")!;
const error = document.querySelector<HTMLParagraphElement>("#error")!;

function cell(text: string, className?: string): HTMLTableCellElement {
  const td = document.createElement("td");
  td.textContent = text;
  if (className) td.className = className;
  return td;
}

function row(link: Link): HTMLTableRowElement {
  const tr = document.createElement("tr");
  const short = document.createElement("a");
  short.href = shortAddress(link);
  short.textContent = link.slug;
  const first = document.createElement("td");
  first.append(short);
  tr.append(first, cell(link.url), cell(String(link.hits), "hits"));
  return tr;
}

function say(problem: unknown): void {
  error.textContent = problem instanceof Error ? problem.message : String(problem);
  error.hidden = false;
}

async function refresh(): Promise<void> {
  rows.replaceChildren(...(await links()).map(row));
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  error.hidden = true;
  const url = new FormData(form).get("url");
  try {
    await shorten(String(url));
    form.reset();
    await refresh();
  } catch (problem) {
    say(problem);
  }
});

refresh().catch(say);
