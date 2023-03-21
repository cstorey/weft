use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId,
    Criterion, Throughput,
};

// Based on the examples in:
// https://github.com/djc/template-benchmarks-rs/tree/b7dee092f621ab5f27d365a1066a499d6f4bf6a2

// classes(): {% if loop.index0 == 0 %}champion{% endif %}

#[derive(weft::WeftRenderable)]
#[template(path = "benches/teams.html")]
struct Teams {
    year: u16,
    teams: Vec<Team>,
}
struct Team {
    name: String,
    score: u8,
}

#[derive(weft::WeftRenderable)]
#[template(path = "benches/big-table.html")]
struct BigTable {
    table: Vec<Vec<usize>>,
}

pub fn teams(b: &mut Criterion) {
    let teams = Teams {
        year: 2015,
        teams: vec![
            Team {
                name: "Jiangsu".into(),
                score: 43,
            },
            Team {
                name: "Beijing".into(),
                score: 27,
            },
            Team {
                name: "Guangzhou".into(),
                score: 22,
            },
            Team {
                name: "Shandong".into(),
                score: 12,
            },
        ],
    };

    b.bench_function("teams", |b| {
        b.iter(|| weft::render_to_string(black_box(&teams)).unwrap())
    });
}

fn big_table(group: &mut BenchmarkGroup<WallTime>, size: usize) {
    let mut table = Vec::with_capacity(size);
    for _ in 0..size {
        let mut inner = Vec::with_capacity(size);
        for i in 0..size {
            inner.push(i);
        }
        table.push(inner);
    }
    let tmpl = BigTable { table };

    let thunk = || weft::render_to_string(&tmpl);
    let bytes = thunk().expect("render").len();
    group.throughput(Throughput::Bytes(bytes as u64));
    group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
        b.iter(thunk)
    });
}

fn big_table_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("big table");
    for size in [1, 4, 16, 64, 256].iter() {
        big_table(&mut group, *size)
    }
    group.finish();
}

fn big_table_reuse_buffer(group: &mut BenchmarkGroup<WallTime>, size: usize) {
    let mut table = Vec::with_capacity(size);
    for _ in 0..size {
        let mut inner = Vec::with_capacity(size);
        for i in 0..size {
            inner.push(i);
        }
        table.push(inner);
    }
    let tmpl = BigTable { table };

    let mut buf = Vec::new();
    weft::render_writer(&tmpl, &mut buf).expect("render");
    // let mut buf = Vec::new();
    // weft::render_writer(&tmpl, &mut buf).expect("render");
    let output_len = buf.len();

    let mut thunk = || {
        buf.clear();
        weft::render_writer(&tmpl, &mut buf)
    };

    group.throughput(Throughput::Bytes(output_len as u64));
    group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
        b.iter(&mut thunk)
    });
}

fn big_table_reuse_buffer_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("big table");
    for size in [1, 4, 16, 64, 256].iter() {
        big_table_reuse_buffer(&mut group, *size)
    }
    group.finish();
}

criterion_group!(
    benches,
    teams,
    big_table_group,
    big_table_reuse_buffer_group,
);
criterion_main!(benches);
