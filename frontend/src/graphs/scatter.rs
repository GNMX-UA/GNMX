use crate::graphs::colors::COLORS;
use crate::GraphData;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use seed::*;
use std::collections::{HashMap, VecDeque};
use std::ops::Range;

pub fn draw(
	canvas_id: &str,
	history: &[(u64, GraphData)],
	range: Range<f64>,
	title: &str,
) -> Option<()> {
	let backend = CanvasBackend::new(canvas_id).expect("cannot find canvas");
	let root = backend.into_drawing_area();
	let font: FontDesc = ("sans-serif", 20.0).into();

	root.fill(&WHITE).ok()?;

	let mut chart = ChartBuilder::on(&root)
		.margin(20)
		.caption(title, font)
		.x_label_area_size(30)
		.y_label_area_size(30)
		.build_cartesian_2d(history.first().unwrap().0..history.last().unwrap().0, range)
		.ok()?;

	// This line will hang if y range is 0.0 .. 0.0, this is a plotters bug probably
	chart
		.configure_mesh()
		.disable_x_mesh()
		.disable_y_mesh()
		.x_labels(10)
		.y_labels(5)
		.draw()
		.ok()?;

	let step = (history.len() / 1000) + 1;

	let iter = history
		.iter()
		.map(|(tick, data)| data.phenotype_sample.iter().map(move |(i, s)| (tick, i, s)))
		.flatten()
		.map(|(t, i, s)| Circle::new((*t, *s), 2, COLORS[i % COLORS.len()].filled()))
		.step_by(step);

	chart.draw_series(iter).ok()?;
	root.present().ok()
}
