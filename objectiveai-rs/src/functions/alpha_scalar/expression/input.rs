use crate::functions;
use indexmap::IndexMap;

pub type ScalarFunctionInputSchema = functions::expression::ObjectInputSchema;

pub mod scalar_function_input_schema {
    use crate::functions;
    pub fn transpile(
        this: super::ScalarFunctionInputSchema,
    ) -> functions::expression::InputSchema {
        functions::expression::InputSchema::Object(this)
    }
}

pub type ScalarFunctionInputExpression = functions::expression::Expression;

pub mod scalar_function_input_expression {
    use crate::functions;
    pub fn transpile(
        this: super::ScalarFunctionInputExpression,
    ) -> functions::expression::WithExpression<
        functions::expression::InputExpression,
    > {
        functions::expression::WithExpression::Expression(this)
    }
}

pub type ScalarFunctionInput = IndexMap<String, functions::expression::Input>;

pub mod scalar_function_input {
    use crate::functions;
    pub fn transpile(
        this: super::ScalarFunctionInput,
    ) -> functions::expression::Input {
        functions::expression::Input::Object(this)
    }
}
