use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "super::ApiConfig::is_none")]
    pub api: Option<super::ApiConfig>,
    #[serde(skip_serializing_if = "super::AuthorizationConfig::is_none")]
    pub authorization: Option<super::AuthorizationConfig>,
    #[serde(skip_serializing_if = "super::AgentsConfig::is_none")]
    pub agents: Option<super::AgentsConfig>,
    #[serde(skip_serializing_if = "super::SwarmsConfig::is_none")]
    pub swarms: Option<super::SwarmsConfig>,
    #[serde(skip_serializing_if = "super::FunctionsConfig::is_none")]
    pub functions: Option<super::FunctionsConfig>,
}

impl Config {
    pub fn api(&mut self) -> &mut super::ApiConfig {
        self.api.get_or_insert_with(super::ApiConfig::default)
    }

    pub fn authorization(&mut self) -> &mut super::AuthorizationConfig {
        self.authorization.get_or_insert_with(super::AuthorizationConfig::default)
    }

    pub fn agents(&mut self) -> &mut super::AgentsConfig {
        self.agents.get_or_insert_with(super::AgentsConfig::default)
    }

    pub fn swarms(&mut self) -> &mut super::SwarmsConfig {
        self.swarms.get_or_insert_with(super::SwarmsConfig::default)
    }

    pub fn functions(&mut self) -> &mut super::FunctionsConfig {
        self.functions.get_or_insert_with(super::FunctionsConfig::default)
    }

    pub fn jq(&self, filter: &str) -> Result<Vec<serde_json::Value>, super::ConfigError> {
        use jaq_core::{load, Compiler, Ctx, FilterT, RcIter};
        use jaq_json::Val;

        let program = load::File { code: filter, path: () };
        let loader = load::Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = load::Arena::default();

        let modules = loader.load(&arena, program)
            .map_err(|errs| super::ConfigError::JqParse(
                errs.into_iter().map(|e| format!("{e:?}")).collect::<Vec<_>>().join(", ")
            ))?;

        let filter = Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|errs| super::ConfigError::JqCompile(
                errs.into_iter().map(|e| format!("{e:?}")).collect::<Vec<_>>().join(", ")
            ))?;

        let input_value = serde_json::to_value(self)
            .map_err(super::ConfigError::Serialize)?;

        let inputs = RcIter::new(core::iter::empty());
        let out = filter.run((Ctx::new([], &inputs), Val::from(input_value)));

        let mut results = Vec::new();
        for item in out {
            match item {
                Ok(val) => results.push(serde_json::Value::from(val)),
                Err(err) => return Err(super::ConfigError::JqRuntime(format!("{err:?}"))),
            }
        }
        Ok(results)
    }
}
