use plotters::prelude::{DrawingBackend, IntoDrawingArea};
use plotters_canvas::CanvasBackend;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::time::Duration;
use wasm_timer::Instant;

use super::ordhelp::*;
use crate::forms::selection::Selection;
use crate::graphs::{line, pheno, environment};
use plotters::prelude::*;
use seed::{prelude::*, *};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphData {
	pub phenotype_variance: f64,
	pub phenotype_distance: f64,
	pub phenotype_sample: Vec<(usize, f64)>, // (patch_index, phenotype)
	pub environment: Vec<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct GraphRanges {
	phenotype_variance: Range<f64>,
	phenotype_distance: Range<f64>,
	phenotype_sample: Range<f64>,
	environment: Range<f64>,
}

pub struct DrawScheduler {
	canvas_id: &'static str,

	ranges: GraphRanges,
	history: Vec<(u64, GraphData)>,

	previous: Option<(Instant, u64, Duration)>,
	stopped: bool,
}

impl DrawScheduler {
	pub fn new(canvas_id: &'static str) -> Self {
		Self {
			canvas_id,
			history: vec![],
			ranges: Default::default(),
			previous: None,
			stopped: false,
		}
	}

	pub fn update_data(&mut self, tick: u64, data: GraphData) -> Option<&'static str> {
		log!("update data");
		if self.stopped {
			self.history.clear();
			self.previous = None;
			self.ranges = Default::default();
			self.stopped = false;
		}

		self.update_ranges(&data);
		self.history.push((tick, data));
		self.maybe_redraw()
	}

	pub fn update_size(&mut self) -> Option<&'static str> {
		log!("update size");
		Self::resize().err()?;
		self.maybe_redraw()
	}

	pub fn stop(&mut self) {
		log!("stopping");
		self.stopped = true
	}

	pub fn view<Msg>(&self) -> Node<Msg> {
		canvas![attrs! {At::Id => "canvas"}]
	}

	fn maybe_redraw(&mut self) -> Option<&'static str> {
		match self.previous {
			Some((instant, tick, duration)) => {
				let expired = instant.elapsed() > duration * 2;
				let new_data = self.history.last().map(|x| x.0).unwrap_or_default() > tick;

				match expired && new_data {
					true => self.draw().err(),
					false => None,
				}
			}
			None => self.draw().err(),
		}
	}

	fn update_ranges(&mut self, data: &GraphData) {
		range_assign(&mut self.ranges.phenotype_variance, data.phenotype_variance);
		range_assign(&mut self.ranges.phenotype_distance, data.phenotype_distance);
		range_slice_assign(
			&mut self.ranges.phenotype_sample,
			data.phenotype_sample.iter().map(|x| x.1),
		);
		range_slice_assign(
			&mut self.ranges.environment,
			data.environment.iter().cloned(),
		);
	}

	fn resize() -> Result<(), &'static str> {
		log!("resizing");
		let canvas = window()
			.document()
			.ok_or("window has no document")?
			.get_element_by_id("canvas")
			.ok_or("could not find canvas element")?
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.map_err(|_| "could not turn canvas into canvas element")?;

		canvas
			.style()
			.set_property("width", "100%")
			.map_err(|_| "Could not resize canvas")?;
		canvas
			.style()
			.set_property("height", "100%")
			.map_err(|_| "Could not resize canvas")?;

		canvas.set_width(canvas.offset_width() as u32);
		// DO NOT REMOVE THIS 10, THE JAVASCRIPT GODS HAVE CHOSEN THIS RANDOM VALUE!!
		canvas.set_height(canvas.offset_height() as u32 - 10);

		Ok(())
	}

	fn draw(&mut self) -> Result<(), &'static str> {
		log!("drawing");
		// We are lazy and just resize the thing before drawing, always
		Self::resize()?;

		let mut canvas = CanvasBackend::new(self.canvas_id).ok_or("cannot find canvas")?;
		let root = canvas.into_drawing_area();
		root.fill(&WHITE).map_err(|_| "could not fill with white")?;

		let mut rows = root.split_evenly((4, 1));

		let mut iter = rows.iter_mut();

		let start = Instant::now();

		let x_range = self.history.first().ok_or("0 elements")?.0..self.history.last().ok_or("0 elements")?.0;

		pheno::draw(
			iter.next().unwrap(),
			&self.history,
			self.ranges.phenotype_sample.clone(),
			"phenotype per patch",
		)
		.ok_or("could not draw phenotype plot")?;

		line::draw(
			iter.next().unwrap(),
			&self.history,
			|data| data.phenotype_variance,
			self.ranges.phenotype_variance.clone(),
			"phenotype variation",
		)
		.ok_or("could not draw phenotype plot")?;

		line::draw(
			iter.next().unwrap(),
			&self.history,
			|data| data.phenotype_distance,
			self.ranges.phenotype_distance.clone(),
			"phenotype variation",
		)
		.ok_or("could not draw phenotype plot")?;

		environment::draw(
			iter.next().unwrap(),
			&self.history,
			self.ranges.environment.clone(),
			"environment per patch",
		)
		.ok_or("could not draw environment plot")?;

		// for index in selection
		// 	.loci
		// 	.iter()
		// 	.enumerate()
		// 	.filter_map(|(index, pred)| pred.then(|| index))
		// {
		// 	// TODO
		// }

		root.present()
			.map(|_| ())
			.map_err(|_| "could not present canvas")?;

		let duration = start.elapsed();
		let last = self.history.last().ok_or("no data to draw")?.0;
		self.previous = Some((start, last, duration));

		Ok(())
	}
}
