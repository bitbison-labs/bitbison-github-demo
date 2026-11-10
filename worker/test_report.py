import unittest

from report import ranked, render

LINKS = [
    {"slug": "0", "url": "https://example.com/a", "hits": 1},
    {"slug": "1", "url": "https://example.com/b", "hits": 3},
    {"slug": "2", "url": "https://example.com/c", "hits": 0},
]


class Report(unittest.TestCase):
    def test_the_most_followed_come_first_with_their_share(self):
        rows = ranked(LINKS, top=2)
        self.assertEqual([row["slug"] for row in rows], ["1", "0"])
        self.assertEqual([row["share"] for row in rows], [0.75, 0.25])

    def test_no_hits_is_no_share(self):
        rows = ranked([{"slug": "0", "url": "https://example.com", "hits": 0}], top=5)
        self.assertEqual(rows[0]["share"], 0.0)

    def test_an_empty_report_says_so(self):
        self.assertEqual(render([]), "no links yet")

    def test_the_table_has_a_line_per_link_under_its_header(self):
        lines = render(ranked(LINKS, top=2)).splitlines()
        self.assertEqual(len(lines), 4)


if __name__ == "__main__":
    unittest.main()
