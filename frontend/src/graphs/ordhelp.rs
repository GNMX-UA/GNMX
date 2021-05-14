use std::ops::Range;

pub fn min_assign(a: &mut f64, b: f64) {
    *a = a.min(b);
}

pub fn max_assign(a: &mut f64, b: f64) {
    *a = a.max(b);
}

pub fn range_assign(a: &mut Range<f64>, b: f64) {
    min_assign(&mut a.start, b);
    max_assign(&mut a.end, b)
}

pub fn range_slice_assign(a: &mut Range<f64>, b: impl Iterator<Item=f64>) {
    b.for_each(|elem| range_assign(a, elem));
}