extern crate levenshtein;

use std::collections::HashSet;
use {ParametricDFA, LevenshteinNFA, Distance};

fn make_distance(n: u8, max_distance: u8) -> Distance {
    if n > max_distance {
        Distance::AtLeast(max_distance + 1u8)
    } else {
        Distance::Exact(n)
    }
}

fn test_levenshtein_nfa_util(left: &str, right: &str) {
    let expected = levenshtein::levenshtein(left, right) as u8;
    for m in 0u8..4u8 {
        let expected_distance = make_distance(expected, m);
        let lev = LevenshteinNFA::levenshtein(m, false);
        test_symmetric(&lev, left, right, expected_distance);
    }
}


fn test_symmetric(lev: &LevenshteinNFA, left: &str, right: &str, expected: Distance) {
    assert_eq!(lev.compute_distance(left, right), expected);
    assert_eq!(lev.compute_distance(right, left), expected);
}


#[test]
fn test_levenshtein() {
    test_levenshtein_nfa_util("abc", "abc");
    test_levenshtein_nfa_util("abc", "abcd");
    test_levenshtein_nfa_util("aab", "ab");
}

fn combinations(alphabet: &[char], len: usize) -> Vec<String> {
    let mut result = vec![];
    let mut prev: Vec<String> = vec![String::from("")];
    for _ in 0..len {
        prev = alphabet
            .iter()
            .cloned()
            .flat_map(|letter: char| {
                prev.iter().map(
                    move |prefix| format!("{}{}", prefix, letter),
                )
            })
            .collect();
        result.extend_from_slice(&prev[..]);
    }
    result
}

#[test]
#[ignore]
fn test_levenshtein_nfa_slow() {
    let test_sample = TestSample::with_num_chars(5);
    test_sample.each(test_levenshtein_nfa_util);
}

fn generate_permutations(current: &mut [char], n: usize, dest: &mut Vec<Vec<char>>) {
    if n == 1 {
        dest.push(Vec::from(current));
    } else {
        for i in 0..n-1 {
            generate_permutations(current, n-1, dest);
            if n % 2 == 0 {
                current.swap(i, n-1);
            } else {
                current.swap(0, n-1);
            }
        }
        generate_permutations(current, n - 1, dest);
    }
}


fn remap(mapping: &[char], text: &String) -> String {
    let mut s = String::new();
    for c in text.as_bytes() {
        let i = c - 97;
        s.push(mapping[i as usize]);
    }
    s
}

#[test]
#[ignore]
fn test_levenshtein_dfa_slow() {
    let test_sample = TestSample::with_num_chars(5);
    let parametric_dfas: Vec<ParametricDFA> = (0u8..4u8)
        .map(|m| {
            let lev = LevenshteinNFA::levenshtein(m, false);
            ParametricDFA::from_nfa(&lev)
        })
        .collect();

    let mut alphabet = vec!['あ', 'b', 'ぃ', 'a', 'え'];
    let mut alternate_mappings = Vec::new();
    generate_permutations(&mut alphabet[..], 5, &mut alternate_mappings);
    
    for mapping in &alternate_mappings {
        for left in test_sample.lefts() {
            let left = remap(mapping, &left);
            for m in 0..4u8 {
                let dfa = parametric_dfas[m as usize].build_dfa(&left);
                for right in test_sample.rights() {
                    let right = remap(mapping, &right);
                    let expected = levenshtein::levenshtein(&left, &right) as u8;
                    let expected_distance = make_distance(expected, m);
                    let result_distance = dfa.eval(&right);
                    assert_eq!(expected_distance, result_distance);
                }
            }
        }
    }
    test_sample.each(test_levenshtein_nfa_util);
}

#[test]
#[ignore]
fn test_levenshtein_parametric_dfa_slow() {
    let parametric_dfas: Vec<ParametricDFA> = (0u8..4u8)
        .map(|m| {
            let lev = LevenshteinNFA::levenshtein(m, false);
            ParametricDFA::from_nfa(&lev)
        })
        .collect();
    let test_sample = TestSample::with_num_chars(5);
    test_sample.each(|left, right| {
        let expected = levenshtein::levenshtein(left, right) as u8;
        for m in 0u8..4u8 {
            let result_distance = parametric_dfas[m as usize].compute_distance(left, right);
            let expected_distance = make_distance(expected, m);
            assert_eq!(expected_distance, result_distance);
        }
    });
}

struct TestSample {
    lefts: Vec<String>,
    rights: Vec<String>,
}

impl TestSample {
    fn with_num_chars(num_chars: usize) -> TestSample {
        let alphabet = vec!['a', 'b', 'c', 'd', 'e'];
        let strings = combinations(&alphabet, num_chars);
        let sorted_strings: Vec<String> = strings
            .iter()
            .filter(|s| {
                let mut v = HashSet::new();
                for c in s.as_bytes() {
                    if !v.contains(c) {
                        let diff = (c - 97) as usize;
                        if diff != v.len() {
                            return false;
                        } else {
                            v.insert(c);
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();
        TestSample {
            lefts: sorted_strings,
            rights: strings,
        }
    }

    fn lefts(&self) -> &[String] {
        &self.lefts
    }

    fn rights(&self) -> &[String] {
        &self.rights
    }

    fn each<F: Fn(&str, &str)>(&self, f: F) {
        for left in &self.lefts {
            for right in &self.rights {
                if left <= right {
                    f(left, right)
                }
            }
        }
    }
}

#[test]
fn test_damerau() {
    let nfa = LevenshteinNFA::levenshtein(2, true);
    test_symmetric(&nfa, "abc", "abc", Distance::Exact(0));
    test_symmetric(&nfa, "abc", "abcd", Distance::Exact(1));
    test_symmetric(&nfa, "abcdef", "abddef", Distance::Exact(1));
    test_symmetric(&nfa, "abcdef", "abdcef", Distance::Exact(1));
}

#[test]
fn test_levenshtein_dfa() {
    let nfa = LevenshteinNFA::levenshtein(2, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa("abcabcaaabc");
    assert_eq!(dfa.num_states(), 317);
}

#[test]
fn test_utf8_simple() {
    let nfa = LevenshteinNFA::levenshtein(1, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa("あ");
    assert_eq!(dfa.eval("あ"), Distance::Exact(0u8));
    assert_eq!(dfa.eval("ぃ"), Distance::Exact(1u8));
}

#[test]
fn test_simple() {
    let q: &str = "abcdef";
    let nfa = LevenshteinNFA::levenshtein(2, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa(q);
    assert_eq!(dfa.eval(q), Distance::Exact(0u8));
    assert_eq!(dfa.eval("abcdf"), Distance::Exact(1u8));
    assert_eq!(dfa.eval("abcdgf"), Distance::Exact(1u8));
    assert_eq!(dfa.eval("abccdef"), Distance::Exact(1u8));
}

#[test]
fn test_jp() {
    let q: &str = "寿司は焦げられない";
    let nfa = LevenshteinNFA::levenshtein(2, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa(q);
    assert_eq!(dfa.eval(q), Distance::Exact(0u8));
    assert_eq!(dfa.eval("寿司は焦げられな"), Distance::Exact(1u8));
    assert_eq!(dfa.eval("寿司は焦げられなI"), Distance::Exact(1u8));
    assert_eq!(
        dfa.eval("寿司は焦げられなIい"),
        Distance::Exact(1u8)
    );
}

#[test]
fn test_jp2() {
    let q: &str = "寿a";
    let nfa = LevenshteinNFA::levenshtein(1, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa(q);
    assert_eq!(dfa.eval(q), Distance::Exact(0u8));
}
