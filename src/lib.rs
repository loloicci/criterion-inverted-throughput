//! # Criterion Inverted Throughput
//! Custom [`criterion::measurement::Measurement`] to get throughputs in the format `[time]/[elements or bytes]`.
//!
//! ## Description
//!
//! With deafult criterion config, result of [throughput measurement](https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#throughput-measurements) is printed like:
//!
//! ```text
//! time:   [2.8617 µs 2.8728 µs 2.8850 µs]
//! thrpt:  [14.558 Melem/s 14.620 Melem/s 14.677 Melem/s]
//! ```
//!
//! Throughput is got in the format `[elements or bytes]/s`.
//! It is fine as a throughput, but sometimes we want to get how much time is
//! cost per 1 element or byte.
//!
//! Using this crate, we can got it in the format `[time]/[element or byte]` without post-processing calculations, like:
//!
//! ```text
//! time:   [2.8581 µs 2.8720 µs 2.8917 µs]
//! thrpt:  [68.849 ns/elem 68.381 ns/elem 68.049 ns/elem]
//! ```
//!
//! ## Usage
//! Specify [`InvertedThroughput`] as your criterion measurement.
//!
//! ```
//! use criterion::{criterion_group, criterion_main, Criterion, Throughput, measurement::Measurement};
//! use criterion_inverted_throughput::InvertedThroughput;
//!
//! fn bench_foo<M: Measurement>(c: &mut Criterion<M>) {
//!     let mut g = c.benchmark_group("foo");
//!
//!     // tell size of input to enable throughput
//!     g.throughput(Throughput::Elements(42u64));
//!
//!     // add benchmarks to the group here like
//!     // g.bench_function("foo", |b| b.iter(|| do_something()));
//!
//!     g.finish();
//! }
//!
//! criterion_group!(
//!     name = Foo;
//!     // specify `InvertedThroughput` as measurement
//!     config = Criterion::default().with_measurement(InvertedThroughput::new());
//!     targets = bench_foo
//! );
//! criterion_main!(Foo);
//! ```

use criterion::measurement::{Measurement, ValueFormatter, WallTime};
use criterion::Throughput;

/// The custom measurement printing inverted throughputs instead of the throughputs
///
/// Specify it as custom measurement in your benchmarks like
/// `Criterion::default().with_measurement(InvertedThroughput::new())`
pub struct InvertedThroughput(WallTime);

impl InvertedThroughput {
    /// Returns a new `InvertedThroughput`
    pub fn new() -> Self {
        InvertedThroughput(WallTime)
    }
}

impl Default for InvertedThroughput {
    fn default() -> Self {
        Self::new()
    }
}

impl Measurement for InvertedThroughput {
    type Intermediate = <WallTime as Measurement>::Intermediate;
    type Value = <WallTime as Measurement>::Value;
    fn start(&self) -> Self::Intermediate {
        self.0.start()
    }
    fn end(&self, i: Self::Intermediate) -> Self::Value {
        self.0.end(i)
    }

    fn add(&self, v1: &Self::Value, v2: &Self::Value) -> Self::Value {
        self.0.add(v1, v2)
    }
    fn zero(&self) -> Self::Value {
        self.0.zero()
    }
    fn to_f64(&self, val: &Self::Value) -> f64 {
        self.0.to_f64(val)
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        self
    }
}

impl InvertedThroughput {
    fn time_per_unit(&self, units: f64, typical_value: f64, values: &mut [f64]) -> &'static str {
        let typical_time = typical_value / units;
        for val in &mut *values {
            let val_per_unit = *val / units;
            *val = val_per_unit;
        }
        self.0.formatter().scale_values(typical_time, values)
    }

    fn static_denom(&self, time_denom: &str, unit_denom: &str) -> &'static str {
        match (unit_denom, time_denom) {
            ("byte", "ps") => "ps/byte",
            ("byte", "ns") => "ns/byte",
            ("byte", "µs") => "µs/byte",
            ("byte", "ms") => "ms/byte",
            ("byte", "s") => "s/byte",
            ("elem", "ps") => "ps/elem",
            ("elem", "ns") => "ns/elem",
            ("elem", "µs") => "µs/elem",
            ("elem", "ms") => "ms/elem",
            ("elem", "s") => "s/elem",
            _ => "UNEXPECTED",
        }
    }
}

impl ValueFormatter for InvertedThroughput {
    fn scale_values(&self, typical_value: f64, values: &mut [f64]) -> &'static str {
        self.0.formatter().scale_values(typical_value, values)
    }

    fn scale_throughputs(
        &self,
        typical_value: f64,
        throughput: &Throughput,
        values: &mut [f64],
    ) -> &'static str {
        let (t_val, t_unit) = match *throughput {
            Throughput::Bytes(v) => (v as f64, "byte"),
            Throughput::BytesDecimal(v) => (v as f64, "byte"),
            Throughput::Elements(v) => (v as f64, "elem"),
        };
        self.static_denom(self.time_per_unit(t_val, typical_value, values), t_unit)
    }

    fn scale_for_machines(&self, values: &mut [f64]) -> &'static str {
        self.0.formatter().scale_for_machines(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[derive(Clone)]
    struct Data {
        typical_value: f64,
        values: Vec<f64>,
        throughput: Throughput,
    }

    impl Data {
        fn new(typical_value: f64, throughput: Throughput) -> Self {
            let mut values: Vec<f64> = vec![];
            for x in -5..5 {
                // generate values in 90%-110% times of typical value
                values.push(typical_value * (1f64 - (x as f64 * 0.02)))
            }
            Self {
                typical_value,
                values,
                throughput,
            }
        }
    }

    enum Unit {
        Element,
        Byte,
        ByteDecimal,
    }

    fn normalize_time(denom: &str, value: f64) -> f64 {
        if denom.to_string().starts_with("ps") {
            value / 1e12
        } else if denom.to_string().starts_with("ns") {
            value / 1e9
        } else if denom.to_string().starts_with("µs") {
            value / 1e6
        } else if denom.to_string().starts_with("ms") {
            value / 1e3
        } else if denom.to_string().starts_with("s") {
            value
        } else {
            panic!("Unexpected denom for time: {}", denom)
        }
    }

    fn normalize_amount(denom: &str, value: f64) -> f64 {
        if denom.to_string().starts_with("G") {
            value * 1e9
        } else if denom.to_string().starts_with("M") {
            value * 1e6
        } else if denom.to_string().starts_with("K") {
            value * 1e3
        } else {
            value
        }
    }

    fn assert_nearly_eq(a: Vec<f64>, b: Vec<f64>) {
        assert_eq!(a.len(), b.len(), "left: {:?} !~= right: {:?}", a, b);
        for i in 0..a.len() {
            assert_ne!(a[i].abs(), 0.0, "left: {:?} !~= right: {:?}", a, b);
            assert!(
                (a[i] - b[i]).abs() < a[i].abs() * 1e-12,
                "left: {:?} !~= right: {:?}",
                a,
                b
            )
        }
    }

    fn assert_nearly_inversion(a: Vec<f64>, b: Vec<f64>) {
        assert_eq!(
            a.len(),
            b.len(),
            "left: {:?} <not inversion> right: {:?}",
            a,
            b
        );
        for i in 0..a.len() {
            assert_ne!(
                a[i].abs(),
                0.0,
                "left: {:?} <not inversion> right: {:?}",
                a,
                b
            );
            assert!(
                (a[i] * b[i] - 1f64).abs() < 0.075,
                "left: {:?} <not inversion> right: {:?} (index: {}, abs(sub(1.0)): {})",
                a,
                b,
                i,
                (a[i] * b[i] - 1f64).abs(),
            )
        }
    }

    #[test_case(Unit::Element, 1, 1e3 ; "test 1 elements")]
    #[test_case(Unit::Element, 10, 1e6 ; "test 10 elements")]
    #[test_case(Unit::Byte, 100, 1e9 ; "test 100 bytes")]
    #[test_case(Unit::ByteDecimal, 1000, 1e12 ; "test 1000 bytesdecimal")]
    #[test_case(Unit::Element, 123, 1.234e15 ; "test 123 elements")]
    #[test_case(Unit::Byte, 123_456_789, 1.234e6 ; "test big bytes")]
    fn test_invert_throughput(unit: Unit, amount: u64, typical_value: f64) {
        // generate test case
        let throughput = match unit {
            Unit::Element => Throughput::Elements(amount),
            Unit::Byte => Throughput::Bytes(amount),
            Unit::ByteDecimal => Throughput::BytesDecimal(amount),
        };
        let data = Data::new(typical_value, throughput.clone());

        // measurements
        let default_measure = WallTime;
        let our_measure = InvertedThroughput(WallTime);

        // compare value with intert throughput
        let mut values_by_default = data.values.clone();
        let mut throughputs_by_default = data.values.clone();
        let mut inverted_throughputs = data.values.clone();

        let unit_by_default = default_measure
            .formatter()
            .scale_values(data.typical_value, &mut values_by_default);
        let unit_by_default_throughputs = default_measure.formatter().scale_throughputs(
            data.typical_value,
            &data.throughput,
            &mut throughputs_by_default,
        );
        let unit_inverted_throughputs = our_measure.scale_throughputs(
            data.typical_value,
            &data.throughput,
            &mut inverted_throughputs,
        );

        let expected_inverted_throuputs: Vec<f64> = values_by_default
            .iter()
            .map(|x| normalize_time(unit_by_default, *x) / amount as f64)
            .collect();
        let normalized_default_throuputs: Vec<f64> = throughputs_by_default
            .iter()
            .map(|x| normalize_amount(unit_by_default_throughputs, *x))
            .collect();
        let normalized_inverted_throuputs: Vec<f64> = inverted_throughputs
            .iter()
            .map(|x| normalize_time(unit_inverted_throughputs, *x))
            .collect();

        assert_nearly_eq(
            expected_inverted_throuputs,
            normalized_inverted_throuputs.clone(),
        );
        assert_nearly_inversion(normalized_inverted_throuputs, normalized_default_throuputs);
    }
}
