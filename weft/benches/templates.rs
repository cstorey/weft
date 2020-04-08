#![feature(test)]

extern crate test;

use std::convert::TryInto;

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

#[bench]
pub fn teams(b: &mut test::Bencher) {
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

    let thunk = || weft::render_to_string(&teams);
    b.bytes = thunk().expect("render").len().try_into().unwrap();

    b.iter(thunk);
}
