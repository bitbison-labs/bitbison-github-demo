"""corral's reporting worker: which links were followed, and how often.

    uv run report.py [--api http://127.0.0.1:8080] [--top 10]

It reads every link from the API and prints the most followed, with each
link's share of all hits.
"""

import argparse
import os
import sys

import httpx
from texttable import Texttable


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
    table = Texttable(max_width=0)
    table.set_deco(Texttable.HEADER)
    table.set_cols_dtype(["t", "i", "t", "t"])
    table.set_cols_align(["l", "r", "r", "l"])
    table.header(["link", "hits", "share", "goes to"])
    for row in rows:
        table.add_row([row["slug"], row["hits"], f"{row['share']:.1%}", row["url"]])
    return table.draw()


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
