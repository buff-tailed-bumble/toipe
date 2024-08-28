use std::io;
use toipe::trie::Trie;

fn main() {
    let mut trie = Trie::new();

    for result in io::stdin().lines() {
        if let Ok(line) = result {
            for word in line.split(char::is_whitespace) {
                if let Err(err) = trie.insert(&word.to_ascii_lowercase()) {
                    println!("{}", err);
                }
            }
        }
    }

    println!("Uncompressed:\n{}", trie);
    if let Ok(compressed) = trie.compress() {
        println!("Compressed:\n{}", compressed);
        for i in 0..compressed.num_words() {
            if let Ok(word) = compressed.sample(i) {
                println!("{}", word)
            }
        }
    }
}
