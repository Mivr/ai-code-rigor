//! gpu-exhaustive: exhaustive 2^32 validation and mutant schemata, CPU vs GPU, via CubeCL.
//!
//! Commands (all print `RESULT ...` lines that results/RESULTS.md is built from):
//!   info            --backend B
//!   spec-validate   (host only: spec vs independent references)
//!   check           --backend B --kernel K [--mutant M] [--stride S --seed N] [--repeat R] [--ept E]
//!   diff            --a B1 --b B2 --kernel K [--mutant M]
//!   mutants         --backend B [--stride S --seed N] [--bound U] [--ept E]
//! Backends: rayon | spec (diff only) | cpu (CubeCL MLIR/LLVM) | gpu (wgpu discrete GPU)
//!           | llvmpipe (wgpu software Vulkan) | default (wgpu default adapter, e.g. Metal)
//! Kernels: bf16 (fast f32->bf16) | bf16-native (CubeCL's own cast) | expf

mod consts;
mod host;
mod kernels;

use consts::*;
use cubecl::prelude::*;
use cubecl::server::Handle;
use cubecl::wgpu::{AutoCompiler, WgpuDevice, WgpuDeviceKind, WgpuRuntime};
use cubecl_runtime::runtime::Runtime as _;
use host::*;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

// ------------------------------------------------------------------------------------------
// CLI
// ------------------------------------------------------------------------------------------

struct Args {
    cmd: String,
    kv: HashMap<String, String>,
}

impl Args {
    fn parse() -> Self {
        let mut it = std::env::args().skip(1);
        let cmd = it.next().unwrap_or_else(|| "help".into());
        let mut kv = HashMap::new();
        while let Some(k) = it.next() {
            let k = k.trim_start_matches("--").to_string();
            let v = it.next().unwrap_or_default();
            kv.insert(k, v);
        }
        Args { cmd, kv }
    }
    fn s(&self, k: &str, d: &str) -> String {
        self.kv.get(k).cloned().unwrap_or_else(|| d.to_string())
    }
    fn u(&self, k: &str, d: u32) -> u32 {
        self.kv.get(k).map(|v| v.parse().expect(k)).unwrap_or(d)
    }
}

fn kind_of(name: &str) -> u32 {
    match name {
        "bf16" => KIND_BF16_FAST,
        "bf16-native" => KIND_BF16_NATIVE,
        "expf" => KIND_EXPF,
        _ => panic!("unknown kernel {name}"),
    }
}

// ------------------------------------------------------------------------------------------
// Backends
// ------------------------------------------------------------------------------------------

enum Be {
    Rayon,
    Spec,
    Cube { client: ComputeClient, label: String },
}

type ComputeClient = cubecl::client::Client;

fn wgpu_client(kind: WgpuDeviceKind) -> ComputeClient {
    // AutoCompiler = SPIR-V on Vulkan when built with `--features spirv`, WGSL otherwise.
    WgpuRuntime::<AutoCompiler>::client(&WgpuDevice::new(kind))
}

fn backend(name: &str) -> Be {
    let client = match name {
        "rayon" => return Be::Rayon,
        "spec" => return Be::Spec,
        #[cfg(feature = "cpu")]
        "cpu" => cubecl::Device::Cpu(Default::default()).client(),
        "gpu" => wgpu_client(WgpuDeviceKind::DiscreteGpu(0)),
        "igpu" => wgpu_client(WgpuDeviceKind::IntegratedGpu(0)),
        "llvmpipe" => wgpu_client(WgpuDeviceKind::Cpu),
        "default" => wgpu_client(WgpuDeviceKind::DefaultDevice),
        _ => panic!("unknown backend {name}"),
    };
    let label = format!("{} on {}", client.name(), client.properties().identity.name);
    Be::Cube { client, label }
}

fn be_label(be: &Be) -> String {
    match be {
        Be::Rayon => format!("rayon ({} threads)", rayon::current_num_threads()),
        Be::Spec => "spec (host)".into(),
        Be::Cube { label, .. } => label.clone(),
    }
}

// ------------------------------------------------------------------------------------------
// Golden table for kernel B (spec = CORE-MATH expf)
// ------------------------------------------------------------------------------------------

struct Golden {
    lo_p: u32,
    hi_p: u32,
    lo_n: u32,
    hi_n: u32,
    chunks: BTreeMap<u32, Vec<u32>>,
}

impl Golden {
    fn empty() -> Self {
        // windows that contain nothing: kernel A never reads them
        Golden { lo_p: 1, hi_p: 0, lo_n: 1, hi_n: 0, chunks: BTreeMap::new() }
    }
}

/// Derive the non-trivial windows from the spec itself (no hand-written thresholds): outside
/// [lo_p, hi_p] positive inputs map to exactly 1.0 (below) or +inf (above); outside
/// [lo_n, hi_n] negative inputs map to exactly 1.0 or +0. Then tabulate the spec for every
/// chunk (2^CHUNK_LOG2 inputs) that intersects a window.
fn build_golden() -> Golden {
    let t0 = Instant::now();
    const BLK: u64 = 1 << 20;
    let parts: Vec<(u32, u32, u32, u32, u64)> = (0..(1u64 << 32) / BLK)
        .into_par_iter()
        .map(|blk| {
            let (mut lp, mut hp, mut ln, mut hn, mut nanbad) = (NONE, 0u32, NONE, 0u32, 0u64);
            for i in 0..BLK {
                let b = (blk * BLK + i) as u32;
                let s = expf_spec(b);
                if b <= F32_INF {
                    if s != ONE_BITS {
                        lp = lp.min(b);
                    }
                    if s != F32_INF {
                        hp = hp.max(b);
                    }
                } else if (0x8000_0000..=0xff80_0000).contains(&b) {
                    if s != ONE_BITS {
                        ln = ln.min(b);
                    }
                    if s != 0 {
                        hn = hn.max(b);
                    }
                } else if (s & 0x7fff_ffff) <= F32_INF {
                    nanbad += 1;
                }
            }
            (lp, hp, ln, hn, nanbad)
        })
        .collect();
    let mut g = Golden { lo_p: NONE, hi_p: 0, lo_n: NONE, hi_n: 0, chunks: BTreeMap::new() };
    let mut nanbad = 0;
    for (lp, hp, ln, hn, nb) in parts {
        g.lo_p = g.lo_p.min(lp);
        g.hi_p = g.hi_p.max(hp);
        g.lo_n = g.lo_n.min(ln);
        g.hi_n = g.hi_n.max(hn);
        nanbad += nb;
    }
    assert_eq!(nanbad, 0, "spec maps some NaN input to a non-NaN");
    let scan = t0.elapsed().as_secs_f64();
    let t1 = Instant::now();
    for c in 0..N_CHUNKS {
        let lo = c << CHUNK_LOG2;
        let hi = lo + (CHUNK as u32 - 1);
        let hits = |a: u32, z: u32| a <= hi && z >= lo;
        if hits(g.lo_p, g.hi_p) || hits(g.lo_n, g.hi_n) {
            let v: Vec<u32> = (0..CHUNK as u32).into_par_iter().map(|i| expf_spec(lo + i)).collect();
            g.chunks.insert(c, v);
        }
    }
    let fill = t1.elapsed().as_secs_f64();
    let win = (g.hi_p - g.lo_p + 1) as u64 + (g.hi_n - g.lo_n + 1) as u64;
    println!(
        "RESULT golden lo_p={:#010x} ({:e}) hi_p={:#010x} ({:e}) lo_n={:#010x} ({:e}) hi_n={:#010x} ({:e}) window_inputs={} chunks={} table_mb={} scan_s={:.3} fill_s={:.3}",
        g.lo_p, f32::from_bits(g.lo_p), g.hi_p, f32::from_bits(g.hi_p),
        g.lo_n, f32::from_bits(g.lo_n), g.hi_n, f32::from_bits(g.hi_n),
        win, g.chunks.len(), g.chunks.len() as u64 * CHUNK * 4 >> 20, scan, fill
    );
    g
}

#[inline]
fn expected_host(kind: u32, b: u32, li: u32, chunk: Option<&Vec<u32>>, g: &Golden) -> u32 {
    if kind != KIND_EXPF {
        return bf16_spec(b);
    }
    if (b & 0x7fff_ffff) > F32_INF {
        F32_NAN_MARK
    } else if (b >= g.lo_p && b <= g.hi_p) || (b >= g.lo_n && b <= g.hi_n) {
        chunk.expect("golden chunk")[li as usize]
    } else if b < 0x8000_0000 {
        if b < g.lo_p { ONE_BITS } else { F32_INF }
    } else if b < g.lo_n {
        ONE_BITS
    } else {
        0
    }
}

#[inline]
fn eval_host(kind: u32, b: u32, m: u32) -> u32 {
    match kind {
        KIND_BF16_FAST => bf16_fast(b),
        KIND_EXPF => expf_fast(b, m),
        _ => panic!("kernel has no plain-Rust twin"),
    }
}

fn vshift_of(kind: u32) -> u32 {
    if kind == KIND_EXPF { 0 } else { 16 }
}

fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

// ------------------------------------------------------------------------------------------
// check: evaluate + compare without moving outputs off the device
// ------------------------------------------------------------------------------------------

struct CheckCfg {
    kind: u32,
    ept: u32,
    stride: u32,
    seed: u64,
}

impl CheckCfg {
    fn threads(&self) -> u32 {
        (CHUNK / (self.ept as u64 * self.stride as u64)) as u32
    }
    fn offset(&self, c: u32) -> u32 {
        if self.stride > 1 { (splitmix(self.seed ^ c as u64) % self.stride as u64) as u32 } else { 0 }
    }
    fn n_inputs(&self) -> u64 {
        self.threads() as u64 * self.ept as u64 * N_CHUNKS as u64
    }
}

struct DevGolden {
    chunks: BTreeMap<u32, (Handle, usize)>,
    dummy: (Handle, usize),
}

fn upload_golden(client: &ComputeClient, g: &Golden) -> (DevGolden, f64) {
    let t0 = Instant::now();
    let mut chunks = BTreeMap::new();
    for (c, v) in &g.chunks {
        chunks.insert(*c, (client.create_from_slice(u32::as_bytes(v)), v.len()));
    }
    let dummy = (client.create_from_slice(u32::as_bytes(&[0u32; 4])), 4);
    // force completion of the uploads before timing anything else
    cubecl::future::block_on(client.sync()).expect("sync");
    (DevGolden { chunks, dummy }, t0.elapsed().as_secs_f64())
}

fn launch_check(
    client: &ComputeClient,
    cfg: &CheckCfg,
    dg: &DevGolden,
    g: &Golden,
    c: u32,
    m: u32,
    threads: u32,
    ept: u32,
) -> Handle {
    let stats = client.empty(threads as usize * STATS_W * 4);
    let (gh, glen) = dg.chunks.get(&c).unwrap_or(&dg.dummy);
    let cube = cube_dim(client, threads);
    unsafe {
        kernels::check_kernel::launch_unchecked(
            client,
            CubeCount::Static(threads / cube, 1, 1),
            CubeDim::new_1d(cube),
            BufferArg::from_raw_parts(gh.clone(), *glen),
            BufferArg::from_raw_parts(stats.clone(), threads as usize * STATS_W),
            c << CHUNK_LOG2,
            ept,
            cfg.stride,
            cfg.offset(c),
            g.lo_p,
            g.hi_p,
            g.lo_n,
            g.hi_n,
            m,
            cfg.kind,
        )
    };
    stats
}

/// Returns (compile+first-dispatch seconds, tiny-dispatch seconds).
fn warm_check(client: &ComputeClient, cfg: &CheckCfg, dg: &DevGolden, g: &Golden) -> (f64, f64) {
    let mut times = vec![];
    for _ in 0..2 {
        let t0 = Instant::now();
        let h = launch_check(client, cfg, dg, g, 0, 0, 256, 1);
        let _ = client.read_one_unchecked(h);
        times.push(t0.elapsed().as_secs_f64());
    }
    (times[0], times[1])
}

fn check_cube(client: &ComputeClient, cfg: &CheckCfg, dg: &DevGolden, g: &Golden, m: u32) -> (Stats, f64) {
    let t0 = Instant::now();
    let threads = cfg.threads();
    let handles: Vec<Handle> =
        (0..N_CHUNKS).map(|c| launch_check(client, cfg, dg, g, c, m, threads, cfg.ept)).collect();
    let bytes = client.read(handles).expect("read stats");
    let secs = t0.elapsed().as_secs_f64();
    let mut s = stats_new();
    let mut covered = 0u64;
    for b in &bytes {
        for rec in u32::from_bytes(b).chunks_exact(STATS_W) {
            stats_merge(&mut s, rec);
            covered += rec[20] as u64;
        }
    }
    coverage_guard(covered, cfg.n_inputs());
    (s, secs)
}

/// A kernel that failed to compile still "returns" zero-filled statistics (CubeCL 0.11.3 logs
/// the compile panic on its server thread and the read succeeds), which reads as a clean pass.
/// Refuse to report anything unless every input was evaluated.
fn coverage_guard(covered: u64, expected: u64) {
    if covered != expected {
        println!("RESULT coverage_error evaluated={covered} expected={expected}");
        std::process::exit(3);
    }
}

fn check_rayon(cfg: &CheckCfg, g: &Golden, m: u32) -> (Stats, f64) {
    let t0 = Instant::now();
    let threads = cfg.threads();
    let vshift = vshift_of(cfg.kind);
    let parts: Vec<Stats> = (0..N_CHUNKS as u64 * threads as u64)
        .into_par_iter()
        .map(|gt| {
            let c = (gt / threads as u64) as u32;
            let t = (gt % threads as u64) as u32;
            let base = c << CHUNK_LOG2;
            let off = cfg.offset(c);
            let chunk = g.chunks.get(&c);
            let mut s = stats_new();
            for j in 0..cfg.ept {
                let li = (t * cfg.ept + j) * cfg.stride + off;
                let b = base + li;
                let v = eval_host(cfg.kind, b, m);
                let e = expected_host(cfg.kind, b, li, chunk, g);
                stats_add(&mut s, b, v, e, vshift);
            }
            s
        })
        .collect();
    let secs = t0.elapsed().as_secs_f64();
    coverage_guard(parts.iter().map(|p| p[20] as u64).sum(), cfg.n_inputs());
    (stats_reduce(parts.iter().map(|p| &p[..])), secs)
}

/// Leanest CPU check for kernel B (max ULP and NaN-ness mismatches only) against the same
/// golden table, as a best-case baseline for the rayon path.
fn expf_minimal_rayon(g: &Golden) -> (u32, u64, f64) {
    let t0 = Instant::now();
    let r = (0..1u32 << 16)
        .into_par_iter()
        .map(|blk| {
            let c = blk >> (CHUNK_LOG2 - 16);
            let chunk = g.chunks.get(&c);
            let (mut mx, mut nm) = (0u32, 0u64);
            for i in 0..1u32 << 16 {
                let b = (blk << 16) | i;
                let li = b - (c << CHUNK_LOG2);
                let v = expf_fast(b, 0);
                let e = expected_host(KIND_EXPF, b, li, chunk, g);
                let (vn, en) = ((v & 0x7fff_ffff) > F32_INF, (e & 0x7fff_ffff) > F32_INF);
                if vn != en {
                    nm += 1;
                } else if !vn {
                    let (x, y) = (ordu(v), ordu(e));
                    mx = mx.max(if x > y { x - y } else { y - x });
                }
            }
            (mx, nm)
        })
        .reduce(|| (0, 0), |a, b| (a.0.max(b.0), a.1 + b.1));
    (r.0, r.1, t0.elapsed().as_secs_f64())
}

// ------------------------------------------------------------------------------------------
// diff: move every output to the host and compare two producers exactly
// ------------------------------------------------------------------------------------------

const OUT_EPT: u32 = 16;

/// Workgroup size: 256, or less where the runtime caps it (the CPU runtime allows 32).
fn cube_dim(client: &ComputeClient, threads: u32) -> u32 {
    threads.min(256).min(client.properties().hardware.max_units_per_cube)
}

fn launch_out(client: &ComputeClient, kind: u32, m: u32, base: u32, n: u32) -> Handle {
    let out = client.empty(n as usize * 4);
    let threads = n / OUT_EPT;
    let cube = cube_dim(client, threads);
    unsafe {
        kernels::out_kernel::launch_unchecked(
            client,
            CubeCount::Static(threads / cube, 1, 1),
            CubeDim::new_1d(cube),
            BufferArg::from_raw_parts(out.clone(), n as usize),
            base,
            OUT_EPT,
            m,
            kind,
        )
    };
    out
}

fn produce(be: &Be, kind: u32, m: u32, c: u32) -> Vec<u32> {
    let base = c << CHUNK_LOG2;
    match be {
        Be::Rayon => (0..CHUNK as u32).into_par_iter().map(|i| eval_host(kind, base + i, m)).collect(),
        Be::Spec => (0..CHUNK as u32)
            .into_par_iter()
            .map(|i| if kind == KIND_EXPF { expf_spec(base + i) } else { bf16_spec(base + i) })
            .collect(),
        Be::Cube { client, .. } => {
            let h = launch_out(client, kind, m, base, CHUNK as u32);
            u32::from_bytes(&client.read_one_unchecked(h)).to_vec()
        }
    }
}

fn warm_out(be: &Be, kind: u32) -> Option<(f64, f64)> {
    if let Be::Cube { client, .. } = be {
        let mut t = vec![];
        for _ in 0..2 {
            let t0 = Instant::now();
            let h = launch_out(client, kind, 0, 0, 256 * OUT_EPT);
            let _ = client.read_one_unchecked(h);
            t.push(t0.elapsed().as_secs_f64());
        }
        Some((t[0], t[1]))
    } else {
        None
    }
}

/// Output class of an f32 bit pattern: 0 zero, 1 subnormal, 2 normal, 3 inf, 4 NaN.
fn class32(v: u32) -> usize {
    let a = v & 0x7fff_ffff;
    match a {
        0 => 0,
        1..=0x007f_ffff => 1,
        0x7f80_0000 => 3,
        x if x > 0x7f80_0000 => 4,
        _ => 2,
    }
}

fn cmd_diff(a: &Args) {
    let kind = kind_of(&a.s("kernel", "bf16"));
    let m = a.u("mutant", 0);
    let ba = backend(&a.s("a", "gpu"));
    let bb = backend(&a.s("b", "spec"));
    let vshift = vshift_of(kind);
    println!("RESULT diff_setup a=\"{}\" b=\"{}\" kernel={} mutant={}", be_label(&ba), be_label(&bb), a.s("kernel", "bf16"), m);
    for (n, be) in [("a", &ba), ("b", &bb)] {
        if let Some((first, second)) = warm_out(be, kind) {
            println!("RESULT jit side={n} first_launch_s={first:.4} second_launch_s={second:.5} jit_estimate_s={:.4}", first - second);
        }
    }
    let (mut ta, mut tb, mut tc) = (0.0, 0.0, 0.0);
    let mut s = stats_new();
    let mut by_class = [0u64; 5];
    let mut examples: Vec<(u32, u32, u32)> = vec![];
    let t_all = Instant::now();
    for c in 0..N_CHUNKS {
        let t0 = Instant::now();
        let va = produce(&ba, kind, m, c);
        ta += t0.elapsed().as_secs_f64();
        let t1 = Instant::now();
        let vb = produce(&bb, kind, m, c);
        tb += t1.elapsed().as_secs_f64();
        let t2 = Instant::now();
        let base = c << CHUNK_LOG2;
        const B: usize = 1 << 16;
        let parts: Vec<(Stats, [u64; 5], Vec<(u32, u32, u32)>)> = va
            .par_chunks(B)
            .zip(vb.par_chunks(B))
            .enumerate()
            .map(|(k, (xa, xb))| {
                let mut st = stats_new();
                let mut cls = [0u64; 5];
                let mut ex = vec![];
                for i in 0..xa.len() {
                    let b = base + (k * B + i) as u32;
                    stats_add(&mut st, b, xa[i], xb[i], vshift);
                    if xa[i] != xb[i] {
                        cls[class32(xb[i] << vshift)] += 1;
                        if ex.len() < 2 {
                            ex.push((b, xa[i], xb[i]));
                        }
                    }
                }
                (st, cls, ex)
            })
            .collect();
        for (st, cls, ex) in parts {
            stats_merge(&mut s, &st);
            for i in 0..5 {
                by_class[i] += cls[i];
            }
            for e in ex {
                if examples.len() < 12 {
                    examples.push(e);
                }
            }
        }
        tc += t2.elapsed().as_secs_f64();
    }
    let total = t_all.elapsed().as_secs_f64();
    let differing: u64 = by_class.iter().sum();
    println!("RESULT diff_stats {}", fmt_stats(&s));
    println!(
        "RESULT diff_summary differing_outputs={} by_class_of_b[zero,subnormal,normal,inf,nan]={:?} a_s={:.3} b_s={:.3} compare_s={:.3} total_s={:.3}",
        differing, by_class, ta, tb, tc, total
    );
    for (b, xa, xb) in examples {
        println!(
            "RESULT diff_example x={:#010x} ({:e}) a={:#010x} b={:#010x}",
            b, f32::from_bits(b), xa, xb
        );
    }
}

// ------------------------------------------------------------------------------------------
// check / mutants commands
// ------------------------------------------------------------------------------------------

struct Ctx {
    be: Be,
    cfg: CheckCfg,
    g: Golden,
    dg: Option<DevGolden>,
}

fn setup(a: &Args, kind: u32) -> Ctx {
    let be = backend(&a.s("backend", "gpu"));
    let stride = a.u("stride", 1);
    let default_ept = if stride == 1 { 1024 } else { 64 };
    let cfg = CheckCfg { kind, ept: a.u("ept", default_ept), stride, seed: a.u("seed", 1) as u64 };
    assert!(cfg.threads() >= 1 && (cfg.threads() <= 32 || cfg.threads() % 256 == 0));
    println!(
        "RESULT setup backend=\"{}\" kernel={} inputs={} ept={} stride={} seed={} threads_per_launch={} launches={}",
        be_label(&be), kind, cfg.n_inputs(), cfg.ept, cfg.stride, cfg.seed, cfg.threads(), N_CHUNKS
    );
    let g = if kind == KIND_EXPF { build_golden() } else { Golden::empty() };
    let dg = if let Be::Cube { client, .. } = &be {
        let (dg, up) = upload_golden(client, &g);
        println!("RESULT golden_upload_s={up:.3}");
        let (first, second) = warm_check(client, &cfg, &dg, &g);
        println!("RESULT jit first_launch_s={first:.4} second_launch_s={second:.5} jit_estimate_s={:.4}", first - second);
        Some(dg)
    } else {
        None
    };
    Ctx { be, cfg, g, dg }
}

fn run_check(ctx: &Ctx, m: u32) -> (Stats, f64) {
    match &ctx.be {
        Be::Rayon => check_rayon(&ctx.cfg, &ctx.g, m),
        Be::Cube { client, .. } => check_cube(client, &ctx.cfg, ctx.dg.as_ref().unwrap(), &ctx.g, m),
        Be::Spec => panic!("spec is not a check backend"),
    }
}

fn cmd_check(a: &Args) {
    let kind = kind_of(&a.s("kernel", "bf16"));
    let m = a.u("mutant", 0);
    let ctx = setup(a, kind);
    if matches!(ctx.be, Be::Rayon) && m == 0 && ctx.cfg.stride == 1 {
        for rep in 0..a.u("repeat", 1) {
            if kind == KIND_BF16_FAST {
                let (n, secs) = bf16_minimal_rayon();
                println!("RESULT check_minimal rep={rep} mismatches={n} secs={secs:.4}");
            } else if kind == KIND_EXPF {
                let (mx, nm, secs) = expf_minimal_rayon(&ctx.g);
                println!("RESULT check_minimal rep={rep} max_ulp={mx} nan_mismatch={nm} secs={secs:.4}");
            }
        }
    }
    for rep in 0..a.u("repeat", 1) {
        let (s, secs) = run_check(&ctx, m);
        println!("RESULT check rep={rep} mutant={m} secs={secs:.4} {}", fmt_stats(&s));
        if kind == KIND_BF16_FAST || kind == KIND_BF16_NATIVE {
            // fixed-order hash of the plain-Rust spec's outputs, same mixing, for cross-checking
            if rep == 0 && ctx.cfg.stride == 1 {
                let t0 = Instant::now();
                let (h1, h2) = (0..1u32 << 16)
                    .into_par_iter()
                    .map(|blk| {
                        let (mut h1, mut h2) = (0u32, 0u32);
                        for i in 0..1u32 << 16 {
                            let b = (blk << 16) | i;
                            let (x, y) = hash_pair(b, bf16_spec(b));
                            h1 = h1.wrapping_add(x);
                            h2 = h2.wrapping_add(y);
                        }
                        (h1, h2)
                    })
                    .reduce(|| (0, 0), |x, y| (x.0.wrapping_add(y.0), x.1.wrapping_add(y.1)));
                println!("RESULT spec_hash hash={h1:08x}{h2:08x} secs={:.3}", t0.elapsed().as_secs_f64());
            }
        }
    }
}

/// Leanest possible CPU check for kernel A (mismatch count only), as a best-case baseline.
fn bf16_minimal_rayon() -> (u64, f64) {
    let t0 = Instant::now();
    let n = (0..1u32 << 16)
        .into_par_iter()
        .map(|blk| {
            (0..1u32 << 16)
                .map(|i| (blk << 16) | i)
                .filter(|&b| bf16_fast(b) != bf16_spec(b))
                .count() as u64
        })
        .sum();
    (n, t0.elapsed().as_secs_f64())
}

fn cmd_mutants(a: &Args) {
    let ctx = setup(a, KIND_EXPF);
    let (s0, secs0) = run_check(&ctx, 0);
    // Oracle: a mutant is killed if it is worse than the original *on the same backend* in the
    // normal-result region or in the subnormal-result region, or flips NaN-ness anywhere.
    let bound = a.u("bound", s0[16]);
    let bound_sub = a.u("bound-sub", s0[18]);
    println!("RESULT mutant id=0 status=original secs={secs0:.4} {}", fmt_stats(&s0));
    println!("RESULT bound max_ulp_normal<={bound} max_ulp_subnormal<={bound_sub} no NaN-ness mismatch");
    let only: Option<u32> = a.kv.get("only").map(|v| v.parse().unwrap());
    let mut total = 0.0;
    let mut killed = 0;
    for &(id, desc) in MUTANTS.iter().skip(1) {
        if only.is_some_and(|o| o != id) {
            continue;
        }
        let (s, secs) = run_check(&ctx, id);
        total += secs;
        let dead = s[16] > bound || s[18] > bound_sub || s[2] > 0;
        killed += dead as u32;
        // identical 64-bit output hash over every evaluated input => same outputs as original
        let same = s[14] == s0[14] && s[15] == s0[15];
        println!(
            "RESULT mutant id={id} status={} same_outputs_as_original={} secs={secs:.4} desc=\"{desc}\" {}",
            if dead { "killed" } else { "survived" },
            if same { "yes" } else { "no" },
            fmt_stats(&s)
        );
    }
    println!("RESULT mutants_summary killed={killed} of={} total_secs={total:.3}", N_MUTANTS);
}

// ------------------------------------------------------------------------------------------
// spec-validate / info
// ------------------------------------------------------------------------------------------

fn cmd_spec_validate() {
    let t0 = Instant::now();
    let bad: u64 = (0..1u32 << 16)
        .into_par_iter()
        .map(|blk| (0..1u32 << 16).filter(|&i| {
            let b = (blk << 16) | i;
            bf16_spec(b) != bf16_oracle(b)
        }).count() as u64)
        .sum();
    println!("RESULT spec_validate_bf16 spec_vs_semantic_oracle_mismatches={bad} secs={:.3}", t0.elapsed().as_secs_f64());

    type R = fn(u32) -> u32;
    for (name, f) in [("f64_exp_rounded", expf_via_f64 as R), ("system_libm_expf", expf_libm as R)] {
        let t0 = Instant::now();
        let parts: Vec<(Stats, Vec<u32>)> = (0..1u64 << 16)
            .into_par_iter()
            .map(|blk| {
                let mut s = stats_new();
                let mut ex = vec![];
                for i in 0..1u32 << 16 {
                    let b = ((blk as u32) << 16) | i;
                    let (v, e) = (f(b), expf_spec(b));
                    stats_add(&mut s, b, v, e, 0);
                    if v != e && (v & 0x7fff_ffff) <= F32_INF && ex.len() < 4 {
                        ex.push(b);
                    }
                }
                (s, ex)
            })
            .collect();
        let s = stats_reduce(parts.iter().map(|p| &p.0[..]));
        let ex: Vec<String> = parts.iter().flat_map(|p| p.1.iter()).take(12).map(|b| format!("{b:#010x}")).collect();
        println!("RESULT spec_validate_expf ref={name} vs=core_math secs={:.3} {}", t0.elapsed().as_secs_f64(), fmt_stats(&s));
        println!("RESULT spec_validate_expf_examples ref={name} {}", ex.join(","));
    }
}

fn cmd_info(a: &Args) {
    let be = backend(&a.s("backend", "gpu"));
    println!("RESULT info backend=\"{}\"", be_label(&be));
    if let Be::Cube { client, .. } = &be {
        let p = client.properties();
        println!("RESULT info_props memory={:?} hardware={:?}", p.memory, p.hardware);
        println!("RESULT info_bf16_uses {:?}", half::bf16::supported_uses(client));
    }
}

/// Probe the backend's floating-point semantics with inputs whose IEEE results are known.
fn cmd_fp_probe(a: &Args) {
    let be = backend(&a.s("backend", "gpu"));
    let Be::Cube { client, label } = &be else { panic!("fp-probe needs a CubeCL backend") };
    let a1 = 1.0f32 + (2.0f32).powi(-12);
    let inp: [f32; 9] = [
        0.3,                         // x
        f32::from_bits(SHIFTER_BITS), // c (runtime copy of 1.5*2^23)
        a1,                           // a
        a1 * a1,                      // z = fl(a*a)
        (2.0f32).powi(-100),          // s1
        (2.0f32).powi(-30),           // s2
        2.5,                          // round(2.5) -> 2 (ties to even)
        -0.5,                         // round(-0.5) -> -0
        -1.75,                        // trunc via cast -> -1
    ];
    let expected: [(&str, f32); 8] = [
        ("(x+c)-c with runtime c [IEEE: 0]", (inp[0] + inp[1]) - inp[1]),
        ("(x+C)-C with literal C [IEEE: 0]", (inp[0] + inp[1]) - inp[1]),
        ("a*a - fl(a*a) [IEEE: 0; FMA-contracted: 2^-24]", a1 * a1 - inp[3]),
        ("2^-100 * 2^-30 [IEEE: 2^-130 subnormal; FTZ: 0]", inp[4] * inp[5]),
        ("2^-100 * 1 [IEEE: 2^-100]", inp[4]),
        ("round(2.5) [ties-to-even: 2]", 2.0),
        ("round(-0.5) [ties-to-even: -0]", -0.0),
        ("f32(i32(-1.75)) [trunc: -1]", -1.0),
    ];
    let hin = client.create_from_slice(f32::as_bytes(&inp));
    let hout = client.empty(8 * 4);
    unsafe {
        kernels::fp_probe::launch_unchecked(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_1d(1),
            BufferArg::from_raw_parts(hin, 9),
            BufferArg::from_raw_parts(hout.clone(), 8),
        )
    };
    let got = f32::from_bytes(&client.read_one_unchecked(hout)).to_vec();
    for (i, (name, e)) in expected.iter().enumerate() {
        let ok = got[i].to_bits() == e.to_bits();
        println!(
            "RESULT fp_probe backend=\"{label}\" case=\"{name}\" got={:e} ({:#010x}) ieee={:e} {}",
            got[i], got[i].to_bits(), e, if ok { "OK" } else { "DIFFERS" }
        );
    }
}

/// Kernel B on hand-picked special inputs, next to the spec.
fn cmd_specials(a: &Args) {
    let be = backend(&a.s("backend", "gpu"));
    let m = a.u("mutant", 0);
    let label = be_label(&be);
    let cases: [(&str, u32); 22] = [
        ("+0", 0x0000_0000),
        ("-0", 0x8000_0000),
        ("+inf", 0x7f80_0000),
        ("-inf", 0xff80_0000),
        ("qNaN", 0x7fc0_0000),
        ("sNaN", 0x7f80_0001),
        ("-qNaN", 0xffc0_0000),
        ("1", 0x3f80_0000),
        ("-1", 0xbf80_0000),
        ("2^-25 (exp rounds to 1)", 0x3300_0000),
        ("2^-24 (first input with exp > 1)", 0x3380_0000),
        ("-2^-25 (exp rounds to 1)", 0xb300_0000),
        ("-2^-25-ulp (first input with exp < 1)", 0xb300_0001),
        ("largest input with finite exp (88.7228)", 0x42b1_7217),
        ("first input with exp = +inf", 0x42b1_7218),
        ("88.3763 (k = 128 path)", 0x42b0_c0a6),
        ("-87.3365 (exp just above FLT_MIN)", 0xc2ae_ac4f),
        ("-87.3365-ulp (exp subnormal)", 0xc2ae_ac50),
        ("-100 (deep subnormal)", 0xc2c8_0000),
        ("most negative input with exp > 0 (-103.97208)", 0xc2cf_f1b4),
        ("first negative input with exp = 0", 0xc2cf_f1b5),
        ("-104", 0xc2d0_0000),
    ];
    let warm = warm_out(&be, KIND_EXPF);
    let _ = warm;
    for (name, x) in cases {
        let v = match &be {
            Be::Rayon => expf_fast(x, m),
            Be::Spec => expf_spec(x),
            Be::Cube { client, .. } => {
                let h = launch_out(client, KIND_EXPF, m, x, 256 * OUT_EPT);
                u32::from_bytes(&client.read_one_unchecked(h))[0]
            }
        };
        let e = expf_spec(x);
        let mut s = stats_new();
        stats_add(&mut s, x, v, e, 0);
        let verdict = if v == e {
            "exact".to_string()
        } else if s[2] > 0 {
            "NaN-ness differs".to_string()
        } else if s[4] > 0 {
            "NaN (payload differs)".to_string()
        } else {
            format!("{} ulp", s[0])
        };
        println!(
            "RESULT special backend=\"{label}\" case=\"{name}\" x={x:#010x} out={v:#010x} ({:e}) spec={e:#010x} ({:e}) verdict=\"{verdict}\"",
            f32::from_bits(v), f32::from_bits(e)
        );
    }
}

fn main() {
    let a = Args::parse();
    match a.cmd.as_str() {
        "specials" => cmd_specials(&a),
        "fp-probe" => cmd_fp_probe(&a),
        "info" => cmd_info(&a),
        "spec-validate" => cmd_spec_validate(),
        "check" => cmd_check(&a),
        "diff" => cmd_diff(&a),
        "mutants" => cmd_mutants(&a),
        _ => {
            eprintln!("see the module docs in src/main.rs for usage");
            std::process::exit(2);
        }
    }
}
