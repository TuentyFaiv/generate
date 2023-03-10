pub static SCHEMA_TS: &str = r#"import { object, string } from "yup";

import type { NAMESchema } from "@typing/schemas/NAME_LOWER"; 

export const NAMEFields = object({
  field: string().required("required")
});

export const DEFAULT_NAME_UPPER_VALUES: NAMESchema = {
  field: ""
};
"#;

pub static SCHEMA: &str = r#"import { object, string } from "yup";

export const NAMEFields = object({
  field: string().required("required")
});

export const DEFAULT_NAME_UPPER_VALUES = {
  field: ""
};
"#;

pub static PROPTYPES: &str = r#"import { NAMEFields } from "@schemas/NAME_LOWER";

import type { InferType } from "yup";

export type NAMESchema = InferType<typeof NAMEFields>;
"#;