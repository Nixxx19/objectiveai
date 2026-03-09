import { ObjectInputSchemaSchema } from "../../expression/input.js";
import { ExpressionSchema } from "../../expression/expression.js";

export const AlphaScalarFunctionInputSchemaSchema = ObjectInputSchemaSchema.meta({ title: "ObjectInputSchema", wrapper: true });
export type AlphaScalarFunctionInputSchema = typeof ObjectInputSchemaSchema;

export const AlphaScalarFunctionInputExpressionSchema = ExpressionSchema.meta({ title: "Expression", wrapper: true });
export type AlphaScalarFunctionInputExpression = typeof ExpressionSchema;
