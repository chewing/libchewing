use std::{
    ffi::{c_char, c_int, CStr},
    rc::Rc,
};

use chewing::conversion::{
    Break, ChewingConversionEngine, ChineseSequence, ConversionEngine, Interval,
};
use chewing_public::types::IntervalType;

use super::{binding::toPreeditBufIndex, types::ChewingData};

#[no_mangle]
pub extern "C" fn InitTree(pgdata: &mut ChewingData, _prefix: *const c_char) -> c_int {
    let dict = unsafe {
        Rc::increment_strong_count(pgdata.dict);
        Rc::from_raw(pgdata.dict)
    };
    pgdata.ce = Some(Box::new(ChewingConversionEngine::new(dict)));
    0
}

#[no_mangle]
pub extern "C" fn TerminateTree(pgdata: &mut ChewingData) {
    pgdata.ce = None;
}

#[no_mangle]
pub extern "C" fn Phrasing(pgdata: &mut ChewingData, _all_phrasing: bool) {
    let ce = pgdata.ce.as_ref().expect("nonnull pointer");
    let syllables_u16 = &pgdata.phone_seq[..pgdata.n_phone_seq as usize];
    let select_strs = pgdata
        .select_str
        .iter()
        .take(pgdata.n_select as usize)
        .map(|it| unsafe { CStr::from_ptr(it.as_ptr()) });
    let select_intervals = pgdata.select_interval.iter().take(pgdata.n_select as usize);
    let breaks = pgdata
        .b_arr_brkpt
        .iter()
        .enumerate()
        .filter(|it| *it.1 == 1)
        .map(|it| Break(it.0))
        .collect();
    let syllables = syllables_u16
        .iter()
        .map(|&syl_u16| syl_u16.try_into().expect("convert u16 to syllable"))
        .collect();

    let selections = select_intervals
        .zip(select_strs)
        .map(|(interval, str)| Interval {
            start: interval.from as usize,
            end: interval.to as usize,
            phrase: str.to_string_lossy().to_string(),
        })
        .collect();

    let sequence = ChineseSequence {
        syllables,
        selections,
        breaks,
    };
    let intervals = match pgdata.phr_out.n_num_cut {
        0 => ce.convert(&sequence),
        _ => ce.convert_next(&sequence, pgdata.phr_out.n_num_cut as usize),
    };

    pgdata.phr_out.n_disp_interval = intervals.len() as c_int;
    for (i, interval) in intervals.into_iter().enumerate() {
        let from = interval.start as c_int;
        let to = interval.end as c_int;
        fill_preedit_buf(pgdata, &interval.phrase, from, to);
        let display_intervals = &mut pgdata.phr_out.disp_interval;
        display_intervals[i].from = from;
        display_intervals[i].to = to;
    }
}

fn fill_preedit_buf(pgdata: &mut ChewingData, phrase: &str, from: c_int, to: c_int) {
    let start = unsafe { toPreeditBufIndex((pgdata as *mut ChewingData).cast(), from) } as usize;
    for i in 0..(to - from) as usize {
        phrase
            .chars()
            .nth(i)
            .unwrap()
            .encode_utf8(&mut pgdata.preedit_buf[start + i].char_);
    }
}

#[no_mangle]
pub extern "C" fn IsIntersect(in1: IntervalType, in2: IntervalType) -> bool {
    in1.from.max(in2.from) < in1.to.min(in2.to)
}
