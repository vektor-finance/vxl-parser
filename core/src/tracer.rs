use nom_tracable::TracableInfo;

#[cfg(feature = "trace")]
pub fn get_tracer() -> TracableInfo {
  TracableInfo::new()
    .backward(true)
    .forward(true)
    .parser_width(40)
    .color(false)
    .fold("term")
}

#[cfg(not(feature = "trace"))]
pub fn get_tracer() -> TracableInfo {
  Default::default()
}
