use seed::{prelude::*, *};

use crate::api::{Config, InitConfig, Suggestion, Suggestions, TempEnum};
use crate::components::Button;
use crate::fields::slider::SliderField;
use crate::fields::{Field, InputField, SelectField};
use seed::futures::StreamExt;

#[derive(Clone, Debug)]
pub enum Msg {
    TMax(<InputField<String> as Field>::Msg),
    PopulationSize(<InputField<u64> as Field>::Msg),
    PopulationType(<SelectField as Field>::Msg),
    EnvironmentSize(<InputField<u64> as Field>::Msg),
    EnvironmentType(<SelectField as Field>::Msg),
}

pub struct InitConfigForm {
    t_max: InputField<u64>,
    population_size: InputField<u64>,
    population_type: SelectField,
    patch_amount: InputField<u64>,
    patch_type: SelectField,
}

fn make_suggestions(names: &[&str]) -> Suggestions {
    names
        .iter()
        .enumerate()
        .map(
            (|(i, s)| Suggestion {
                name: s.to_string(),
                value: i as i64,
            }),
        )
        .collect()
}

impl InitConfigForm {
    pub fn new() -> Self {
        let pop_types = make_suggestions(&["uniform", "normal", "equal"]);
        let env_type = make_suggestions(&["uniform", "normal", "equal"]);

        Self {
            t_max: InputField::new("Ticks", false)
                .with_placeholder("leave empty to run indefinitely")
                .with_initial(Some(100_000)),
            population_size: InputField::new("Population size", false).with_initial(Some(6_000)),
            population_type: SelectField::new("Type", pop_types, false),
            patch_amount: InputField::new("Patch amount", false).with_initial(Some(2)),
            patch_type: SelectField::new("Type", env_type, false),
        }
    }

    pub fn update(&mut self, msg: Msg, orders: &mut impl Orders<Msg>) -> bool {
        // so much copy pasta but oh well
        match msg {
            Msg::TMax(msg) => self.t_max.update(msg, &mut orders.proxy(Msg::TMax)),
            Msg::PopulationSize(msg) => self
                .population_size
                .update(msg, &mut orders.proxy(Msg::PopulationSize)),
            Msg::PopulationType(msg) => self
                .population_type
                .update(msg, &mut orders.proxy(Msg::PopulationType)),
            Msg::EnvironmentSize(msg) => self
                .patch_amount
                .update(msg, &mut orders.proxy(Msg::EnvironmentSize)),
            Msg::EnvironmentType(msg) => self
                .patch_type
                .update(msg, &mut orders.proxy(Msg::EnvironmentType)),
        }
    }

    fn extract(&self) -> Option<InitConfig> {
        let t_max = self.t_max.value(true);
        let population_size = self.population_size.value(true);
        let population_type = self.population_type.value(true);
        let patch_amount = self.patch_amount.value(true);
        let patch_type = self.patch_type.value(true);

        // Some(InitConfig {
        // 	t_max,
        // 	population_size: population_size?,
        // 	population_type: population_type?,
        // 	patch_amount: patch_amount?,
        // 	patch_type: patch_type?,
        // })

        Some(InitConfig {
            t_max,
            kind: TempEnum::Default,
        })
    }

    pub fn view(&self, disabled: bool) -> Node<Msg> {
        div![
			div![
				C!["columns"],
				div![
					C!["column"],
					self.population_size
						.view(disabled)
						.map_msg(Msg::PopulationSize)
				],
				div![
					C!["column is-narrow"],
					self.population_type
						.view(disabled)
						.map_msg(Msg::PopulationType)
				],
			],
			div![
				C!["columns"],
				div![
					C!["column"],
					self.patch_amount.view(disabled).map_msg(Msg::EnvironmentSize)
				],
				div![
					C!["column is-narrow"],
					self.patch_type.view(disabled).map_msg(Msg::EnvironmentType)
				],
			],
		]
    }
}
