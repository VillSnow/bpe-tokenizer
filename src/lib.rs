use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::AddAssign,
};
use trie_rs::{Trie, TrieBuilder};

pub struct BpeModel<C: Eq + Hash + Ord + Clone> {
    tokens: HashSet<Vec<C>>,
    tokens_builder: TrieBuilder<C>,
    tokens_trie: Trie<C>,
}

enum ScannerBody<'data, C: Eq + Hash + Ord + Clone> {
    First {
        chars: HashSet<&'data C>,
    },
    Merge {
        pairs: HashMap<(&'data [C], &'data [C]), usize>,
    },
}

pub struct Scanner<'model, 'data, C: Eq + Hash + Ord + Clone> {
    parent: &'model mut BpeModel<C>,
    body: ScannerBody<'data, C>,
}

impl<C: Eq + Hash + Ord + Clone> BpeModel<C> {
    pub fn new() -> Self {
        let tokens_builder = TrieBuilder::new();
        let tokens_trie = tokens_builder.build();
        Self {
            tokens: HashSet::new(),
            tokens_builder,
            tokens_trie,
        }
    }

    pub fn scanner(&mut self) -> Scanner<C> {
        let body = if self.tokens.is_empty() {
            ScannerBody::First {
                chars: HashSet::new(),
            }
        } else {
            ScannerBody::Merge {
                pairs: HashMap::new(),
            }
        };
        Scanner { parent: self, body }
    }

    pub fn tokenized<'a>(&self, word: &'a [C]) -> Vec<&'a [C]> {
        let mut tokenized = Vec::new();

        let mut word = word;
        while !word.is_empty() {
            let hit_len = self
                .tokens_trie
                .common_prefix_search(word)
                .into_iter()
                .map(|x| x.len())
                .max()
                .unwrap_or(1);

            tokenized.push(&word[0..hit_len]);
            word = &word[hit_len..];
        }

        tokenized
    }
}

impl<'model, 'data, C: Eq + Hash + Ord + Clone> Scanner<'model, 'data, C> {
    pub fn scan(&mut self, word: &'data [C], freq: usize) {
        match &mut self.body {
            ScannerBody::First { chars } => {
                chars.extend(word);
            }
            ScannerBody::Merge { pairs } => {
                let tokenized = self.parent.tokenized(word);

                for (&a, &b) in tokenized.iter().zip(tokenized.iter().skip(1)) {
                    pairs.entry((a, b)).or_default().add_assign(freq);
                }
            }
        }
    }

    pub fn finish(self) {
        match self.body {
            ScannerBody::First { chars } => {
                for c in chars {
                    let t = vec![c.clone()];
                    self.parent.tokens.insert(t.clone());
                    self.parent.tokens_builder.push(t);
                }
            }
            ScannerBody::Merge { pairs } => {
                let best_pair = pairs.into_iter().max_by_key(|&(_, v)| v).unwrap().0;
                let mut token = Vec::new();
                token.extend(best_pair.0.into_iter().cloned());
                token.extend(best_pair.1.into_iter().cloned());

                if !self.parent.tokens.contains(&token) {
                    self.parent.tokens.insert(token.clone());
                }
                self.parent.tokens_builder.push(token);
                self.parent.tokens_trie = self.parent.tokens_builder.build();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case01() {
        let mut model = BpeModel::new();

        let data = "ABCDCDABCDCDE".chars().collect::<Vec<_>>();
        let data = data.as_slice();

        for _ in 0..5 {
            let mut scanner = model.scanner();
            scanner.scan(data, 1);
            scanner.finish();
        }

        let tokenized = model
            .tokenized(data)
            .into_iter()
            .map(|cs| cs.iter().copied().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ");
        dbg!(tokenized);
    }
}
