#![feature(test)]

extern crate test;

use chewing::{
    conversion::{ChewingEngine, Composition, Symbol},
    dictionary::{Dictionary, TrieBuf},
    syl,
    zhuyin::Bopomofo::*,
};
use test::Bencher;

fn test_dictionary() -> impl Dictionary {
    TrieBuf::from([
        (vec![syl![H, A]], vec![("哈", 1)]),
        (vec![syl![H, A], syl![H, A]], vec![("哈哈", 1)]),
    ])
}

#[bench]
fn bench_conv(b: &mut Bencher) {
    let dict = test_dictionary();
    let engine = ChewingEngine::new();
    let mut composition = Composition::new();
    for _ in 0..40 {
        composition.push(Symbol::from(syl![H, A]));
    }
    b.iter(|| engine.convert(&dict, &composition).nth(1));
}
