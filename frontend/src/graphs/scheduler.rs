use plotters::prelude::{DrawingBackend, IntoDrawingArea};
use plotters_canvas::CanvasBackend;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::time::Duration;
use wasm_timer::Instant;

use super::ordhelp::*;
use crate::graphs::{environment, line, loci, pheno};
use plotters::coord::Shift;
use plotters::prelude::*;
use seed::{prelude::*, *};

#[derive(Clone, Debug)]
pub enum Tab {
	General,
	Loci,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphData {
	pub phenotype_variance: f64,
	pub phenotype_distance: f64,
	pub phenotype_sample: Vec<(usize, f64)>, // (patch_index, phenotype)
	pub environment: Vec<f64>,
	pub loci: Vec<Vec<(usize, f64)>>,
}

#[derive(Clone, Debug, Default)]
pub struct GraphRanges {
	phenotype_variance: Range<f64>,
	phenotype_distance: Range<f64>,
	phenotype_sample: Range<f64>,
	environment: Range<f64>,
	loci: Vec<Range<f64>>,
}

pub struct DrawScheduler {
	canvas_id: &'static str,

	ranges: GraphRanges,
	history: Vec<(u64, GraphData)>,

	previous: Option<(Instant, u64, Duration)>,
	stopped: bool,
	tab: Tab,
}


impl DrawScheduler {
	pub fn new(canvas_id: &'static str) -> Self {
		Self {
			canvas_id,
			history: vec![],
			ranges: Default::default(),
			previous: None,
			stopped: false,
			tab: Tab::General,
		}
	}

	pub fn update(&mut self, msg: Tab, _: &mut impl Orders<Tab>) {
		self.tab = msg
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

	pub fn update_size(&mut self) -> Option<&'static str> {
		Self::resize().err()?;
		self.maybe_redraw()
	}

	pub fn stop(&mut self) {
		self.stopped = true
	}

	fn view_tabs(&self) -> Node<Tab> {
		// FIXME: this is terrible
		let which = match self.tab {
			Tab::General => true,
			Tab::Loci => false,
		};

		ul![
			li![
				IF!(which => C!["is-active"]),
				a![
					mouse_ev(Ev::Click, |_| Tab::General),
					span![C!["icon is-small"], i![C!["fas fa-list"]]],
					span!["General"]
				]
			],
			li![
				IF!(!which => C!["is-active"]),
				a![
					mouse_ev(Ev::Click, |_| Tab::Loci),
					span![C!["icon is-small"], i![C!["fas fa-microscope"]]],
					span!["Loci"]
				]
			],
		]
	}

	pub fn view(&self) -> Vec<Node<Tab>> {
		vec![
			div![C!["tabs is-centered is-boxed"], self.view_tabs()],
			canvas![attrs! {At::Id => "canvas"}],
		]
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
		self.ranges.loci.resize(data.loci.len(), Range::default());

		for index in 0..data.loci.len() {
			range_slice_assign(
				&mut self.ranges.loci[index],
				data.loci[index].iter().map(|x| x.1),
			)
		}
	}

	fn resize() -> Result<(), &'static str> {
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
			.set_property("height", "85%")
			.map_err(|_| "Could not resize canvas")?;

		canvas.set_width(canvas.offset_width() as u32);
		// DO NOT REMOVE THIS 169, THE JAVASCRIPT GODS HAVE CHOSEN THIS RANDOM VALUE!!
		canvas.set_height(canvas.offset_height() as u32);

		Ok(())
	}

	fn draw_general_tab(
		&self,
		root: &mut DrawingArea<CanvasBackend, Shift>,
	) -> Result<(), &'static str> {
		let mut rows = root.split_evenly((4, 1));
		pheno::draw(
			&mut rows[0],
			&self.history,
			self.ranges.phenotype_sample.clone(),
			"phenotype per patch",
		)
		.ok_or("could not draw phenotype plot")?;

		line::draw(
			&mut rows[1],
			&self.history,
			|data| data.phenotype_variance,
			self.ranges.phenotype_variance.clone(),
			"phenotype variation",
		)
		.ok_or("could not draw phenotype plot")?;

		line::draw(
			&mut rows[2],
			&self.history,
			|data| data.phenotype_distance,
			self.ranges.phenotype_distance.clone(),
			"phenotype variation",
		)
		.ok_or("could not draw phenotype plot")?;

		environment::draw(
			&mut rows[3],
			&self.history,
			self.ranges.environment.clone(),
			"environment per patch",
		)
		.ok_or("could not draw environment plot")?;

		Ok(())
	}

	fn draw_loci_tab(
		&self,
		root: &mut DrawingArea<CanvasBackend, Shift>,
	) -> Result<(), &'static str> {
		let loci = self.history.first().ok_or("No data to draw.")?.1.loci.len();
		let mut rows = root.split_evenly((loci, 1));

		for locus in 0..loci {
			loci::draw(
				&mut rows[locus],
				&self.history,
				locus,
				self.ranges.loci[locus].clone(),
				format!("locus #{}", locus).as_str(),
			)
			.ok_or("could not draw loci plot")?;
		}

		Ok(())
	}

	fn draw(&mut self) -> Result<(), &'static str> {
		// We are lazy and just resize the thing before drawing, always
		Self::resize()?;

		let start = Instant::now();

		let mut canvas = CanvasBackend::new(self.canvas_id).ok_or("cannot find canvas")?;
		let mut root = canvas.into_drawing_area();
		root.fill(&WHITE).map_err(|_| "could not fill with white")?;

		match self.tab {
			Tab::General => self.draw_general_tab(&mut root),
			Tab::Loci => self.draw_loci_tab(&mut root),
		}?;

		root.present()
			.map(|_| ())
			.map_err(|_| "could not present canvas")?;

		let duration = start.elapsed();
		let last = self.history.last().ok_or("no data to draw")?.0;
		self.previous = Some((start, last, duration));

		Ok(())
	}
}
