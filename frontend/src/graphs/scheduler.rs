use plotters_canvas::CanvasBackend;
use std::ops::Range;
use std::time::Duration;
use wasm_timer::Instant;
use serde::{Serialize, Deserialize};
use plotters::prelude::{DrawingBackend, IntoDrawingArea};

use crate::forms::selection::Selection;
use crate::graphs::{line, scatter};
use super::ordhelp::*;
use seed::{prelude::*, *};
use plotters::prelude::*;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphData {
	pub population: u64,
	pub phenotype_variance: f64,
	pub phenotype_distance: f64,
	pub phenotype_sample: Vec<(usize, f64)>, // (patch_index, phenotype)
}

#[derive(Clone, Debug, Default)]
pub struct GraphRanges {
	population: Range<f64>,
	phenotype_variance: Range<f64>,
	phenotype_distance: Range<f64>,
	phenotype_sample: Range<f64>,
}

pub struct DrawScheduler {
	canvas_id: &'static str,

	selection: Option<Selection>,
	ranges: GraphRanges,
	history: Vec<(u64, GraphData)>,

	previous: Option<(Instant, u64)>,
	stopped: bool
}

impl DrawScheduler {
	pub fn new(canvas_id: &'static str) -> Self {
		Self {
			canvas_id,
			history: vec![],
			ranges: Default::default(),
			selection: None,
			previous: None,
			stopped: false
		}
	}

	pub fn update_data(&mut self, tick: u64, data: GraphData) -> Option<&'static str> {
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

	pub fn update_selection(&mut self, selection: Selection) -> Option<&'static str> {
		self.selection = Some(selection);
		self.maybe_redraw()
	}

	pub fn update_size(&mut self) -> Option<&'static str> {
		Self::resize().err()?;
		self.maybe_redraw()
	}

	pub fn stop(&mut self) {
		self.stopped = true
	}

	fn maybe_redraw(&mut self) -> Option<&'static str> {
		match self.previous {
			Some((instant, tick)) => {
				let expired = instant.elapsed() > Duration::from_millis(100);
				let new_data = self.history.last().map(|x| x.0).unwrap_or_default() > tick;

				match expired && new_data
				{
					true => self.draw().err(),
					false => None,
				}
			}
			None => self.draw().err()
		}
	}

	fn update_ranges(&mut self, data: &GraphData) {
		range_assign(&mut self.ranges.population, data.population as f64);
		range_assign(&mut self.ranges.phenotype_variance, data.phenotype_variance);
		range_assign(&mut self.ranges.phenotype_distance, data.phenotype_distance);
		range_slice_assign(
			&mut self.ranges.phenotype_sample,
			data.phenotype_sample.iter().map(|x| x.1),
		);
	}

	fn count_selection(selection: &Selection) -> usize {
		let mut count = 0;
		count += selection.phenotypes as usize;
		count += selection.variance as usize;
		count += selection.distance as usize;
		count += selection.environment as usize;
		count += selection.loci.iter().filter(|x| **x).count();

		count
	}

	fn resize() -> Result<(), &'static str> {
		let document = window().document().ok_or("window has no document")?;

		let width = document
			.get_element_by_id("main")
			.ok_or("could not find element canvasses")?
			.dyn_into::<web_sys::HtmlDivElement>()
			.map_err(|_| "could not turn canvasses into div element")?
			.offset_width() as u32;

		document
			.get_element_by_id("canvas")
			.ok_or("could not find canvas element")?
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.map_err(|_| "could not turn canvas into canvas element")?
			.set_width(width);

		Ok(())
	}

	fn draw(&mut self) -> Result<(), &'static str> {
		let selection = self.selection.as_ref().ok_or("no selection")?;
		let count = Self::count_selection(selection);

		let mut canvas = CanvasBackend::new(self.canvas_id).ok_or("cannot find canvas")?;
		let root = canvas.into_drawing_area();
		root.fill(&WHITE).map_err(|_| "could not fill with white")?;

		let mut rows = root.split_evenly((count, 1));

		let mut iter = rows.iter_mut();

		self.previous = Some((
			Instant::now(),
			self.history.last().ok_or("no data to draw")?.0,
		));

		if selection.phenotypes {
			scatter::draw(
				iter.next().unwrap(),
				&self.history,
				self.ranges.phenotype_sample.clone(),
				"phenotype per patch",
			)
			.ok_or("could not draw phenotype plot")?
		}

		if selection.variance {
			line::draw(
				iter.next().unwrap(),
				&self.history,
				|data| data.phenotype_variance,
				self.ranges.phenotype_variance.clone(),
				"phenotype variation",
			)
			.ok_or("could not draw phenotype plot")?
		}

		if selection.distance {
			line::draw(
				iter.next().unwrap(),
				&self.history,
				|data| data.phenotype_distance,
				self.ranges.phenotype_distance.clone(),
				"phenotype variation",
			)
			.ok_or("could not draw phenotype plot")?
		}

		for index in selection
			.loci
			.iter()
			.enumerate()
			.filter_map(|(index, pred)| pred.then(|| index))
		{
			// TODO
		}

		root.present().map(|_|()).map_err(|_| "could not present canvas")?;

		Ok(())
	}
}
