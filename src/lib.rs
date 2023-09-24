use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    hash::Hash,
    ops::AddAssign,
};
use trie_rs::{Trie, TrieBuilder};

struct VocabChar<C> {
    char: C,
    token_head: Cell<usize>,
}

pub struct Vocab<C: Ord + Hash + Clone> {
    words: Vec<Vec<VocabChar<C>>>,
    tokens: HashSet<Vec<C>>,
}

pub struct Tokenizer<C: Ord + Hash + Clone> {
    trie: Trie<C>,
}

impl<C: Ord + Hash + Clone> Vocab<C> {
    pub fn new<Words, Word>(words: Words) -> Self
    where
        Words: IntoIterator<Item = Word>,
        Word: IntoIterator<Item = C>,
    {
        Self {
            words: words
                .into_iter()
                .map(|w| {
                    w.into_iter()
                        .map(|char| VocabChar {
                            char,
                            token_head: Cell::new(1),
                        })
                        .collect::<Vec<_>>()
                })
                .filter(|w| !w.is_empty())
                .collect(),
            tokens: HashSet::new(),
        }
    }

    pub fn merge(&mut self, min_freq: usize) -> Result<(), ()> {
        let mut pairs = HashMap::<Vec<C>, Vec<&VocabChar<C>>>::new();
        for word in &self.words {
            let mut a_pos = 0;
            loop {
                let a_len = word[a_pos].token_head.get();
                let b_pos = a_pos + a_len;
                if b_pos >= word.len() {
                    break;
                }
                let b_len = word[b_pos].token_head.get();

                let token = word[a_pos..][..a_len + b_len]
                    .iter()
                    .map(|x| x.char.clone())
                    .collect();

                pairs.entry(token).or_default().push(&word[a_pos]);
                a_pos = b_pos;
            }
        }

        let best = pairs
            .into_iter()
            .filter(|(_, v)| v.len() >= min_freq)
            .max_by_key(|(_, v)| v.len());
        let best = best.ok_or(())?;

        for a in best.1 {
            a.token_head.set(best.0.len());
        }
        self.tokens.insert(best.0);

        Ok(())
    }

    pub fn build(&self) -> Tokenizer<C>
    where
        C: Ord,
    {
        let mut builder = TrieBuilder::new();
        for x in &self.tokens {
            builder.push(x)
        }
        Tokenizer {
            trie: builder.build(),
        }
    }

    pub fn tokens(&self) -> &HashSet<Vec<C>> {
        &self.tokens
    }
}

impl<C: Ord + Hash + Clone> Tokenizer<C> {
    pub fn tokenize<'a>(&self, mut word: &'a [C]) -> Vec<&'a [C]> {
        let mut result = Vec::new();
        while !word.is_empty() {
            let n = self
                .trie
                .common_prefix_search(word)
                .into_iter()
                .map(|x| x.len())
                .max()
                .unwrap_or(1);
            result.push(&word[..n]);
            word = &word[n..];
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use super::*;

    #[test]
    fn case01() {
        let data = "ABCDCDABCDCDE".chars().collect::<Vec<_>>();
        let mut vocab = Vocab::new([data.clone()]);
        for _ in 0..4 {
            vocab.merge();
        }

        let tokenizer = vocab.build();

        let tokenized = tokenizer
            .tokenize(&data)
            .into_iter()
            .map(|cs| cs.iter().copied().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ");
        dbg!(tokenized);
    }

    #[test]
    fn case02() {
        let mut vocab = Vocab::new(
            BufReader::new(File::open("data.txt").unwrap())
                .lines()
                .filter_map(|x| x.map(|x| x.chars().collect::<Vec<_>>()).ok()),
        );

        for _ in 0..100 {
            vocab.merge()
        }

        let tokenizer = vocab.build();

        let test_words = BufReader::new(File::open("data.txt").unwrap())
            .lines()
            .take(10)
            .map(|x| x.map(|x| x.chars().collect::<Vec<_>>()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        for word in &test_words {
            let tokens = tokenizer.tokenize(&word);

            let display = tokens
                .into_iter()
                .map(|cs| cs.iter().copied().collect::<String>())
                .collect::<Vec<_>>()
                .join(" ");
            println!("{}", display)
        }
    }
}
