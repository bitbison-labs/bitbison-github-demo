"""corral's reporting worker: which links were followed, and how often.

    uv run report.py [--api http://127.0.0.1:8080] [--top 10]

It reads every link from the API and prints the most followed, with each
link's share of all hits.
"""

import argparse
import os
import sys

import httpx


def ranked(links: list[dict], top: int) -> list[dict]:
    """The `top` most followed links, each with its share of all hits."""
    total = sum(link["hits"] for link in links)
    by_hits = sorted(links, key=lambda link: (-link["hits"], link["slug"]))
    return [
        {**link, "share": link["hits"] / total if total else 0.0}
        for link in by_hits[:top]
    ]


def render(rows: list[dict]) -> str:
    if not rows:
        return "no links yet"
    width = max(len(row["slug"]) for row in rows)
    lines = [f"{'link':<{width}}  {'hits':>6}  {'share':>6}  goes to"]
    for row in rows:
        lines.append(
            f"{row['slug']:<{width}}  {row['hits']:>6}  {row['share']:>6.1%}  {row['url']}"
        )
    return "\n".join(lines)


def fetch(api: str) -> list[dict]:
    response = httpx.get(f"{api.rstrip('/')}/links", timeout=10.0)
    response.raise_for_status()
    return response.json()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--api", default=os.environ.get("CORRAL_API", "http://127.0.0.1:8080")
    )
    parser.add_argument("--top", type=int, default=10)
    args = parser.parse_args(argv)
    try:
        links = fetch(args.api)
    except httpx.HTTPError as error:
        print(f"corral-worker: {error}", file=sys.stderr)
        return 1
    print(render(ranked(links, args.top)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
