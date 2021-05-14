use crate::fields::toggle::ToggleField;
use crate::fields::Field;
use seed::{prelude::*, *};

#[derive(Clone, Debug)]
pub struct Selection {
	pub phenotypes: bool,
	pub variance: bool,
	pub distance: bool,
	pub environment: bool,
	pub loci: Vec<bool>,
}

#[derive(Clone, Debug)]
pub enum Msg {
	Phenotypes(<ToggleField as Field>::Msg),
	Variance(<ToggleField as Field>::Msg),
	Distance(<ToggleField as Field>::Msg),
	Environment(<ToggleField as Field>::Msg),
	Loci(<ToggleField as Field>::Msg, usize),
}

pub struct SelectionForm {
	phenotypes: ToggleField,
	variance: ToggleField,
	distance: ToggleField,
	environment: ToggleField,
	loci: Vec<ToggleField>,
}

impl SelectionForm {
	pub fn new() -> Self {
		Self {
			phenotypes: ToggleField::new("phenotypes", true),
			variance: ToggleField::new("phenotypes", true),
			distance: ToggleField::new("phenotypes", true),
			environment: ToggleField::new("phenotypes", true),
			loci: vec![]
		}
	}

	pub fn set_loci(&mut self, amount: usize) {
		self.loci = (0..amount)
			.map(|i| ToggleField::new(format!("locus #{}", i).as_str(), false))
			.collect()
	}

	pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> Selection {
		match msg {
			Msg::Phenotypes(msg) => self.phenotypes.update(msg, &mut orders.proxy(Msg::Phenotypes)),
			Msg::Variance(msg) => self.variance.update(msg, &mut orders.proxy(Msg::Variance)),
			Msg::Distance(msg) => self.distance.update(msg, &mut orders.proxy(Msg::Distance)),
			Msg::Environment(msg) => self.environment.update(msg, &mut orders.proxy(Msg::Environment)),
			Msg::Loci(msg, index) => self.loci[index].update(msg, &mut orders.proxy(move |msg| Msg::Loci(msg, index)))
		};

		self.extract()
	}

	pub fn extract(&self) -> Selection {
		Selection {
			phenotypes: self.phenotypes.value(true).unwrap(),
			variance: self.variance.value(true).unwrap(),
			distance: self.distance.value(true).unwrap(),
			environment: self.environment.value(true).unwrap(),
			loci: self.loci.iter().map(|locus| locus.value(true).unwrap()).collect()
		}
	}

	pub fn view(&self) -> Node<Msg> {
		div![
			self.phenotypes.view(false).map_msg(Msg::Phenotypes),
			self.variance.view(false).map_msg(Msg::Variance),
			self.distance.view(false).map_msg(Msg::Distance),
			self.environment.view(false).map_msg(Msg::Environment),
			self.loci.iter().enumerate().map(|(i, elem)| elem.view(false).map_msg(move |msg| Msg::Loci(msg, i))),
		]
	}
}
