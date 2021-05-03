use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use seed::*;

pub fn draw(canvas_id: &str, values: &[f32]) -> Option<()> {
    log!(canvas_id);
    let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
    let root = backend.into_drawing_area();
    let font: FontDesc = ("sans-serif", 20.0).into();

    root.fill(&WHITE).ok()?;
    let min = values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max = values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(min, max)
        .ok()?;

    chart.configure_mesh().x_labels(3).y_labels(3).draw().ok()?;

    chart
        .draw_series(LineSeries::new(values.iter().enumerate(), &RED))
        .ok()?;

    root.present().ok()
}
