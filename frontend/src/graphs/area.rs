use crate::api::State;
use crate::graphs::colors::COLORS;
use crate::GraphData;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use seed::*;
use std::collections::HashMap;

pub fn draw(
	canvas_id: &str,
	history: &[(u64, GraphData)],
	map: impl Fn(&GraphData) -> f64,
) -> Option<()> {
	let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
	let root = backend.into_drawing_area();
	let font: FontDesc = ("sans-serif", 20.0).into();

	let x_max = history.last()?.0;

	let y_max = history
		.iter()
		.map(|(_, data)| map(data))
		.max_by(|a, b| a.partial_cmp(b).unwrap())?;

	root.fill(&WHITE).ok()?;

	let mut chart = ChartBuilder::on(&root)
		.margin(20)
		.x_label_area_size(30)
		.y_label_area_size(30)
		.build_cartesian_2d(0..x_max, 0.0..y_max)
		.ok()?;

	// This line will hang if y range is 0.0 .. 0.0, this is a plotters bug probably
	chart.configure_mesh().x_labels(3).y_labels(3).draw().ok()?;

	chart
		.draw_series(AreaSeries::new(
			history.iter().map(|(tick, data)| (*tick, map(data))),
			0.0,
			&COLORS[5],
		))
		.ok()?;

	root.present().ok()
}
