use crate::graphs::colors::COLORS;
use crate::GraphData;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use seed::*;
use std::collections::HashMap;

pub fn draw(canvas_id: &str, history: &[(u64, GraphData)]) -> Option<()> {
	let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
	let root = backend.into_drawing_area();
	let font: FontDesc = ("sans-serif", 20.0).into();

	let x_max = history.last()?.0;

	let y_max = history
		.iter()
		.map(|(_, data)| data.phenotype_sample.iter().map(|(_, p)| p))
		.flatten()
		.max_by(|a, b| a.partial_cmp(b).unwrap())?;

	root.fill(&WHITE).ok()?;

	let mut chart = ChartBuilder::on(&root)
		.margin(20)
		.x_label_area_size(30)
		.y_label_area_size(30)
		.build_cartesian_2d(0..x_max, 0.0..*y_max)
		.ok()?;

	// This line will hang if y range is 0.0 .. 0.0, this is a plotters bug probably
	chart.configure_mesh().x_labels(10).y_labels(3).draw().ok()?;

	let iter = history
		.iter()
		.map(|(tick, data)| data.phenotype_sample.iter().map(move |(i, s)| (tick, i, s)))
		.flatten()
		// .map(|(t, i, s)| Rectangle::new([(*t, *s), (t + 1, s + 0.05)], COLORS[i % COLORS.len()].filled()));
		.map(|(t, i, s)| Circle::new((*t, *s), 2, COLORS[i % COLORS.len()].filled()));

	chart.draw_series(iter).ok()?;
	root.present().ok()
}
