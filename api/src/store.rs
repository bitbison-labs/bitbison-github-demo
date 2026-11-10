//! The links, in memory, behind one lock.

use std::sync::Mutex;

use serde::Serialize;

const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Link {
    pub slug: String,
    pub url: String,
    pub hits: u64,
}

#[derive(Default)]
pub struct Store {
    links: Mutex<Vec<Link>>,
}

/// The nth link's slug: n in base 62, so slugs stay short and never collide.
fn slug_of(mut n: usize) -> String {
    let mut out = Vec::new();
    loop {
        out.push(ALPHABET[n % ALPHABET.len()]);
        n /= ALPHABET.len();
        if n == 0 {
            break;
        }
    }
    out.reverse();
    String::from_utf8(out).expect("the alphabet is ASCII")
}

impl Store {
    pub fn add(&self, url: String) -> Link {
        let mut links = self.links.lock().unwrap();
        let link = Link {
            slug: slug_of(links.len()),
            url,
            hits: 0,
        };
        links.push(link.clone());
        link
    }

    /// Every link, most followed first.
    pub fn all(&self) -> Vec<Link> {
        let mut links = self.links.lock().unwrap().clone();
        links.sort_by(|a, b| b.hits.cmp(&a.hits).then_with(|| a.slug.cmp(&b.slug)));
        links
    }

    /// Counts a hit and gives the URL to redirect to.
    pub fn hit(&self, slug: &str) -> Option<String> {
        let mut links = self.links.lock().unwrap();
        let link = links.iter_mut().find(|l| l.slug == slug)?;
        link.hits += 1;
        Some(link.url.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_count_in_base_62() {
        assert_eq!(slug_of(0), "0");
        assert_eq!(slug_of(61), "Z");
        assert_eq!(slug_of(62), "10");
    }

    #[test]
    fn a_hit_is_counted_and_orders_the_list() {
        let store = Store::default();
        store.add("https://example.com/a".into());
        let b = store.add("https://example.com/b".into());
        assert_eq!(store.hit(&b.slug).as_deref(), Some("https://example.com/b"));
        assert_eq!(store.hit("missing"), None);
        let all = store.all();
        assert_eq!(all[0].slug, b.slug);
        assert_eq!(all[0].hits, 1);
    }
}
