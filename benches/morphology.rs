use criterion::{Criterion, black_box, criterion_group, criterion_main};
use vlazba::jvozba::{
    jvokaha::jvokaha,
    jvozba,
    tools::{analyze_lujvo_spelling, search_selrafsi_from_rafsi2, RafsiOptions},
};

fn opts() -> RafsiOptions<'static> {
    RafsiOptions {
        exp_rafsi: true,
        custom_cmavo: None,
        custom_cmavo_exp: None,
        custom_gismu: None,
        custom_gismu_exp: None,
    }
}

fn bench_morphology(c: &mut Criterion) {
    let options = opts();
    let pair = vec!["klama".to_string(), "gasnu".to_string()];
    let triple = vec![
        "melbi".to_string(),
        "cmalu".to_string(),
        "zdani".to_string(),
    ];

    c.bench_function("jvozba_best_only_pair", |b| {
        b.iter(|| jvozba(black_box(&pair), false, true, true, &options))
    });

    c.bench_function("jvozba_full_pair", |b| {
        b.iter(|| jvozba(black_box(&pair), false, true, false, &options))
    });

    c.bench_function("jvozba_best_only_triple", |b| {
        b.iter(|| jvozba(black_box(&triple), false, true, true, &options))
    });

    c.bench_function("jvokaha_bramlatu", |b| {
        b.iter(|| jvokaha(black_box("bramlatu")).unwrap())
    });

    c.bench_function("analyze_bardymlatu", |b| {
        b.iter(|| analyze_lujvo_spelling(black_box("bardymlatu"), &options).unwrap())
    });

    c.bench_function("reverse_rafsi_zuk", |b| {
        b.iter(|| search_selrafsi_from_rafsi2(black_box("zuk"), &options).unwrap())
    });
}

criterion_group!(benches, bench_morphology);
criterion_main!(benches);
