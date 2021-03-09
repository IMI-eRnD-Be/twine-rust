use criterion::{criterion_group, criterion_main, Criterion};
use std::path::Path;

fn generation(strs: &[&str], output_file: &Path) {
    twine::build_translations_from_str(strs, output_file).unwrap();
}

fn generation_benchmark(c: &mut Criterion) {
    let out_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OUT_DIR", out_dir.as_ref().as_os_str());
    let output_file = Path::new("test");
    let strs: Vec<_> = (0..1000)
        .map(|i| {
            format!(
                r#"
                [band_tool_{}]
                    en = Tool
                    fr = Outil
                "#,
                i,
            )
        })
        .collect();
    c.bench_function("generation", |b| {
        b.iter(|| {
            generation(
                &strs.iter().map(|x| x.as_str()).collect::<Vec<_>>(),
                &output_file,
            )
        })
    });
}

criterion_group!(benches, generation_benchmark);
criterion_main!(benches);
