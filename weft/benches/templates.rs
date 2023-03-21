use criterion::{black_box, criterion_group, criterion_main, Criterion};

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

macro_rules! big_table_sized {
    ($name: ident, $inner: ident, $size: expr) => {
        pub fn $name(b: &mut Criterion) {
            $inner(b, $size);
        }
    };
}

big_table_sized!(big_table_allocating_001x001, big_table, 1);
big_table_sized!(big_table_allocating_004x004, big_table, 4);
big_table_sized!(big_table_allocating_016x016, big_table, 16);
big_table_sized!(big_table_allocating_064x064, big_table, 64);
big_table_sized!(big_table_allocating_256x256, big_table, 256);
big_table_sized!(big_table_reuse_buffer_001x001, big_table_reuse_buffer, 1);
big_table_sized!(big_table_reuse_buffer_004x004, big_table_reuse_buffer, 4);
big_table_sized!(big_table_reuse_buffer_016x016, big_table_reuse_buffer, 16);
big_table_sized!(big_table_reuse_buffer_064x064, big_table_reuse_buffer, 64);
big_table_sized!(big_table_reuse_buffer_256x256, big_table_reuse_buffer, 256);

fn big_table(b: &mut Criterion, size: usize) {
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
    // b.bytes = thunk().expect("render").len().try_into().unwrap();

    b.bench_function(&format!("big table {size}"), |b| b.iter(thunk));
}

fn big_table_reuse_buffer(b: &mut Criterion, size: usize) {
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
    // b.bytes = buf.len().try_into().unwrap();

    let mut thunk = || {
        buf.clear();
        weft::render_writer(&tmpl, &mut buf)
    };

    b.bench_function(&format!("big table (reused buffer) {size}"), |b| {
        b.iter(&mut thunk)
    });
}

criterion_group!(
    benches,
    teams,
    big_table_allocating_001x001,
    big_table_allocating_004x004,
    big_table_allocating_016x016,
    big_table_allocating_064x064,
    big_table_allocating_256x256,
    big_table_reuse_buffer_001x001,
    big_table_reuse_buffer_004x004,
    big_table_reuse_buffer_016x016,
    big_table_reuse_buffer_064x064,
    big_table_reuse_buffer_256x256,
);
criterion_main!(benches);
