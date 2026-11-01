//! Opt-in synthetic measurements; elapsed times are evidence, never CI gates.
use rstest::rstest;
use std::{fmt::Write as _, hint::black_box, time::Instant};
use taloon::resolution::Resolution;

#[derive(Clone, Copy, Debug)]
enum Shape {
    Chain,
    Fanout,
}
fn fixture(shape: Shape, count: usize) -> String {
    let mut lock = "version=1\nrevision=5\n[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='p00000'}]\n".to_owned();
    for i in 0..count {
        writeln!(lock, "[[package]]\nname='p{i:05}'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}").unwrap();
        let width = match shape {
            Shape::Chain => 1,
            Shape::Fanout => 4,
        };
        let end = (i + 1 + width).min(count);
        if i + 1 < end {
            lock.push_str("dependencies=[");
            for next in i + 1..end {
                write!(lock, "{{name='p{next:05}'}},").unwrap();
            }
            lock.push_str("]\n");
        }
    }
    lock
}
#[rstest]
#[case::chain(Shape::Chain, 1500)]
#[case::fanout(Shape::Fanout, 3000)]
#[ignore = "opt-in release measurement; no elapsed-time assertion"]
fn resolution_and_shortest_path_cost(#[case] shape: Shape, #[case] count: usize) {
    // Arrange: every external node is retained even without supported archives.
    let lock = fixture(shape, count);
    for sample in 1..=5 {
        // Act
        let started = Instant::now();
        let resolution = Resolution::from_uv_lock(black_box(&lock)).unwrap();
        let parse_us = started.elapsed().as_micros();
        let last = resolution.targets().last().unwrap().package();
        let started = Instant::now();
        let paths = resolution.paths(black_box(last));
        let path_us = started.elapsed().as_micros();
        // Assert: the workload and shortest path remain the same between revisions.
        assert_eq!(resolution.targets().len(), count);
        assert_eq!(paths.len(), 1);
        let expected_depth = match shape {
            Shape::Chain => count,
            Shape::Fanout => (count - 1).div_ceil(4) + 1,
        };
        assert_eq!(paths[0].steps.len(), expected_depth);
        println!(
            "shape={shape:?} packages={count} sample={sample} parse_us={parse_us} path_us={path_us}"
        );
    }
}
