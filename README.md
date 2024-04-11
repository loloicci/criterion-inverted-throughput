# Criterion Inverted Throughput
Custom [criterion](https://github.com/bheisler/criterion.rs) measurement to get thropughts in the format `[elements or bytes]/[time]`

## Description
With deafult criterion config, result of benchmarks for throughput is printed like:

```
time:   [1.5726 µs 1.5727 µs 1.5728 µs]
thrpt:  [26.704 Melem/s 26.706 Melem/s 26.707 Melem/s]
```

Throughput is got in the format `[elements or bytes]/s`.
It is fine as a throughput, but sometimes we want to get how much time is
cost per 1 element or byte.

Using this crate, we can got it in the format `[cost time]/[element or byte]` without post-processing calculations, like:

```
time:   [1.5876 µs 1.5980 µs 1.6078 µs]
thrpt:  [69.903 ns/elem 69.479 ns/elem 69.027 ns/elem]
```

## Usage
Specify `InvertedThroughput` as the measurement in your benchmarks.

### Example
```
criterion_group!(
    name = Fum;
    // specify `InvertedThroughput` as measurement
    config = Criterion::default().with_measurement(InvertedThroughput::new());
    targets = bench_foo
);
criterion_main!(Foo);
```
