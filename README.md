# Criterion Inverted Throughput
Custom [criterion](https://github.com/bheisler/criterion.rs) measurement to get thropughts in the format `[elements or bytes]/[time]`

## Description
With deafult criterion config, result of [throughput measurement](https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#throughput-measurements) is printed like:

```text
time:   [2.8617 µs 2.8728 µs 2.8850 µs]
thrpt:  [14.558 Melem/s 14.620 Melem/s 14.677 Melem/s]
```

Throughput is got in the format `[elements or bytes]/s`.
It is fine as a throughput, but sometimes we want to get how much time is
cost per 1 element or byte.

Using this crate, we can got it in the format `[time]/[element or byte]` without post-processing calculations, like:

```text
time:   [2.8581 µs 2.8720 µs 2.8917 µs]
thrpt:  [68.849 ns/elem 68.381 ns/elem 68.049 ns/elem]
```

## Usage
Specify `InvertedThroughput` as the measurement in your benchmarks.

### Example
```rust
criterion_group!(
    name = Fum;
    // specify `InvertedThroughput` as measurement
    config = Criterion::default().with_measurement(InvertedThroughput::new());
    targets = bench_foo
);
criterion_main!(Foo);
```
