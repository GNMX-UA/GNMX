use crate::graphs::constants::*;
use crate::GraphData;
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use seed::*;
use std::collections::{HashMap, VecDeque};
use std::ops::Range;

pub fn draw(
	backend: &mut DrawingArea<CanvasBackend, Shift>,
	history: &[(u64, GraphData)],
	y_range: Range<f64>,
	title: &str,
) -> Option<()> {
	let font: FontDesc = ("sans-serif", 20.0).into();
	let x_range = history.first()?.0..history.last()?.0;

	let mut chart = ChartBuilder::on(&backend)
		.margin(20)
		.caption(title, font)
		.x_label_area_size(30)
		.y_label_area_size(30)
		.build_cartesian_2d(x_range, y_range)
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

	let step = (history.len() / MAX_COLS) + 1;

	let iter = history
		.iter()
		.map(|(t, d)| {
			d.environment
				.iter()
				.enumerate()
				.map(move |(i, s)| (t, i, s))
		})
		.flatten()
		.map(|(t, i, s)| Circle::new((*t, *s), 2, COLORS[i % COLORS.len()].filled()))
		.step_by(step);

	chart.draw_series(iter).ok()?;

	Some(())
}
