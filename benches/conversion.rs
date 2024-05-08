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
        // (vec![syl![G, U, O, TONE2]], vec![("國", 1)]),
        // (vec![syl![M, I, EN, TONE2]], vec![("民", 1)]),
        // (vec![syl![D, A, TONE4]], vec![("大", 1)]),
        // (vec![syl![H, U, EI, TONE4]], vec![("會", 1)]),
        // (vec![syl![D, AI, TONE4]], vec![("代", 1)]),
        // (vec![syl![B, I, AU, TONE3]], vec![("表", 1), ("錶", 1)]),
        // (
        //     vec![syl![G, U, O, TONE2], syl![M, I, EN, TONE2]],
        //     vec![("國民", 200)],
        // ),
        // (
        //     vec![syl![D, A, TONE4], syl![H, U, EI, TONE4]],
        //     vec![("大會", 200)],
        // ),
        // (
        //     vec![syl![D, AI, TONE4], syl![B, I, AU, TONE3]],
        //     vec![("代表", 200), ("戴錶", 100)],
        // ),
        // (vec![syl![X, I, EN]], vec![("心", 1)]),
        // (vec![syl![K, U, TONE4], syl![I, EN]], vec![("庫音", 300)]),
        // (
        //     vec![syl![X, I, EN], syl![K, U, TONE4], syl![I, EN]],
        //     vec![("新酷音", 200)],
        // ),
        // (
        //     vec![syl![C, E, TONE4], syl![SH, TONE4], syl![I, TONE2]],
        //     vec![("測試儀", 42)],
        // ),
        // (
        //     vec![syl![C, E, TONE4], syl![SH, TONE4]],
        //     vec![("測試", 9318)],
        // ),
        // (
        //     vec![syl![I, TONE2], syl![X, I, A, TONE4]],
        //     vec![("一下", 10576)],
        // ),
        // (vec![syl![X, I, A, TONE4]], vec![("下", 10576)]),
        (vec![syl![H, A]], vec![("哈", 1)]),
        (vec![syl![H, A], syl![H, A]], vec![("哈哈", 1)]),
    ])
}

#[bench]
fn bench_conv_fast_path(b: &mut Bencher) {
    let dict = test_dictionary();
    let engine = ChewingEngine::new();
    let mut composition = Composition::new();
    for _ in 0..40 {
        composition.push(Symbol::from(syl![H, A]));
    }
    b.iter(|| engine.convert(&dict, &composition).nth(0));
}

#[bench]
fn bench_conv_slow_path(b: &mut Bencher) {
    let dict = test_dictionary();
    let engine = ChewingEngine::new();
    let mut composition = Composition::new();
    for _ in 0..40 {
        composition.push(Symbol::from(syl![H, A]));
    }
    b.iter(|| engine.convert(&dict, &composition).nth(1));
}
