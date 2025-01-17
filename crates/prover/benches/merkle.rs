#![feature(iter_array_chunks)]

use criterion::Criterion;

#[cfg(target_arch = "x86_64")]
pub fn cpu_merkle(c: &mut criterion::Criterion) {
    use itertools::Itertools;
    use num_traits::Zero;
    use stwo_prover::core::backend::avx512::AVX512Backend;
    use stwo_prover::core::backend::{CPUBackend, Col};
    use stwo_prover::core::fields::m31::BaseField;
    use stwo_prover::core::vcs::ops::MerkleOps;
    use stwo_prover::platform;

    const N_COLS: usize = 1 << 8;
    const LOG_SIZE: u32 = 16;
    let cols = (0..N_COLS)
        .map(|_| {
            (0..(1 << LOG_SIZE))
                .map(|_| BaseField::zero())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("merkle throughput");
    group.throughput(criterion::Throughput::Elements((N_COLS << LOG_SIZE) as u64));
    group.throughput(criterion::Throughput::Bytes(
        (N_COLS << (LOG_SIZE + 2)) as u64,
    ));
    group.bench_function("cpu merkle", |b| {
        b.iter(|| {
            CPUBackend::commit_on_layer(LOG_SIZE, None, &cols.iter().collect_vec());
        })
    });

    if !platform::avx512_detected() {
        return;
    }
    let cols = (0..N_COLS)
        .map(|_| {
            (0..(1 << LOG_SIZE))
                .map(|_| BaseField::zero())
                .collect::<Col<AVX512Backend, BaseField>>()
        })
        .collect::<Vec<_>>();

    group.bench_function("avx merkle", |b| {
        b.iter(|| {
            AVX512Backend::commit_on_layer(LOG_SIZE, None, &cols.iter().collect_vec());
        })
    });
}

#[cfg(target_arch = "x86_64")]
criterion::criterion_group!(
    name=merkle;
    config = Criterion::default().sample_size(10);
    targets=cpu_merkle);
criterion::criterion_main!(merkle);
