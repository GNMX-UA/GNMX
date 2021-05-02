use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use seed::*;

pub fn draw(canvas_id: &str, values: &[f32]) -> Option<()> {
    log!(canvas_id);
    let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
    let root = backend.into_drawing_area();
    let font: FontDesc = ("sans-serif", 20.0).into();

    root.fill(&WHITE).ok()?;

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .caption(format!("y=x^{}", power), font)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-1f32..1f32, -1.2f32..1.2f32)
        .ok()?;

    chart.configure_mesh().x_labels(3).y_labels(3).draw().ok()?;

    chart
        .draw_series(LineSeries::new(values.iter().enumerate(), &RED))
        .ok()?;

    root.present().ok()
}
