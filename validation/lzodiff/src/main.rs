//! Differential validation: our `lzo` vs the independent GPL `rust-lzo`
//! (Linux lzo1x_decompress_safe, converted to Rust). Local-only oracle.
use std::path::Path;

fn rl_decode(src: &[u8], cap: usize) -> Option<Vec<u8>> {
    let mut dst = vec![0u8; cap];
    let (out, err) = rustlzo::LZOContext::decompress_to_slice(src, &mut dst);
    let n = out.len();
    matches!(err, rustlzo::LZOError::OK).then(|| dst[..n].to_vec())
}
fn ours_decode(src: &[u8], cap: usize) -> Option<Vec<u8>> {
    let mut dst = vec![0u8; cap];
    lzo::decompress_into(src, &mut dst).ok().map(|n| dst[..n].to_vec())
}

fn main() {
    let rw = std::env::var("RW").unwrap_or_else(|_| "../rw".into());
    let d = Path::new(&rw);

    // ── Part A: real liblzo2 corpus — both decode to the exact original ──
    let mut names: Vec<String> = std::fs::read_dir(d).unwrap().filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".raw")).map(|n| n.trim_end_matches(".raw").to_string()).collect();
    names.sort();
    let (mut a_ok, mut a_bytes) = (0u64, 0u64);
    for name in &names {
        let raw = std::fs::read(d.join(format!("{name}.raw"))).unwrap();
        for algo in ["1","15","999"] {
            let p = d.join(format!("{name}.{algo}.lzo"));
            if !p.exists() { continue; }
            let lzo = std::fs::read(&p).unwrap();
            assert_eq!(ours_decode(&lzo, raw.len()).as_deref(), Some(&raw[..]), "OURS wrong {name}.{algo}");
            assert_eq!(rl_decode(&lzo, raw.len()).as_deref(), Some(&raw[..]), "rust-lzo wrong {name}.{algo}");
            a_ok += 1; a_bytes += raw.len() as u64;
        }
    }
    println!("Part A (real liblzo2 corpus): {a_ok} blocks, {a_bytes} bytes — ours == rust-lzo == original, byte-exact");

    // ── Part B: mutation fuzz of real blocks — near-valid inputs where both
    //    decoders often succeed; outputs must agree, ours must never panic ──
    // Seeds: small real lzo blocks (the ~/src/lzo opcode probes + small corpus).
    let mut seeds: Vec<Vec<u8>> = Vec::new();
    let lzo_data = Path::new("/Users/4n6h4x0r/src/lzo/tests/data");
    for n in ["hello","run_a","pattern","farmatch","empty"] {
        if let Ok(b) = std::fs::read(lzo_data.join(format!("{n}.lzo"))) { seeds.push(b); }
    }
    for n in ["readme","librs","one","empty"] {
        for a in ["1","999"] {
            if let Ok(b) = std::fs::read(d.join(format!("{n}.{a}.lzo"))) { seeds.push(b); }
        }
    }
    println!("Part B seeds: {}", seeds.len());
    const CAP: usize = 1 << 18;
    let n_inputs = 3_000_000u32;
    let (mut both_ok, mut both_err, mut split, mut disagree) = (0u64,0u64,0u64,0u64);
    for it in 0..n_inputs {
        let mut s = it.wrapping_mul(2_654_435_761).wrapping_add(0x9E37_79B9);
        let mut nx = || { s ^= s << 13; s ^= s >> 17; s ^= s << 5; s };
        let seed = &seeds[(nx() as usize) % seeds.len()];
        let mut buf = seed.clone();
        if !buf.is_empty() {
            let k = (nx() % 4) + 1; // mutate 1..4 bytes
            for _ in 0..k {
                let i = (nx() as usize) % buf.len();
                buf[i] = (nx() >> 24) as u8;
            }
            // occasionally truncate/extend
            if nx() % 8 == 0 && buf.len() > 1 { buf.truncate((nx() as usize) % buf.len() + 1); }
        }
        let ours = ours_decode(&buf, CAP);
        let theirs = rl_decode(&buf, CAP);
        match (ours.is_some(), theirs.is_some()) {
            (true,true) => { both_ok += 1; if ours != theirs { disagree += 1; if disagree<=3 { eprintln!("DISAGREE seed-mut len={} ours={:?} theirs={:?}", buf.len(), ours.as_ref().map(|v|v.len()), theirs.as_ref().map(|v|v.len())); } } }
            (false,false) => both_err += 1,
            _ => split += 1,
        }
    }
    println!("Part B (mutation fuzz, {n_inputs} inputs): both_ok={both_ok} both_err={both_err} one-only={split} | DISAGREEMENTS-when-both-OK={disagree}");
    println!("\nVERDICT: {}", if disagree==0 {"no output divergence on any input where both decoders succeeded; ours never panicked"} else {"DIVERGENCE FOUND — investigate"});
}
