use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use seed::*;
use crate::api::State;

pub fn draw(canvas_id: &str, history: &[State]) -> Option<()> {
    let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
    let root = backend.into_drawing_area();
    let font: FontDesc = ("sans-serif", 20.0).into();

    let mapper = |state: &State| state.patches.len();

    let x_max = history.last()?.tick;
    let y_max = history.iter().map(mapper).max()?;

    root.fill(&WHITE).ok()?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0..x_max, 0..y_max)
        .ok()?;

    chart.configure_mesh().x_labels(3).y_labels(3).draw().ok()?;

    chart
        .draw_series(AreaSeries::new(
            history.iter().map(|state| (state.tick, mapper(state))),
            0,
            &RED,
        ))
        .ok()?;

    root.present().ok()
}
