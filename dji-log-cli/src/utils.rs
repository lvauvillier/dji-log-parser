pub(crate) fn decimal_to_dms(decimal: f64) -> (f64, f64, f64) {
    let degrees = decimal.trunc();
    let minutes = ((decimal - degrees) * 60.0).trunc();
    let seconds = (decimal - degrees - minutes / 60.0) * 3600.0;
    (degrees, minutes, seconds)
}
