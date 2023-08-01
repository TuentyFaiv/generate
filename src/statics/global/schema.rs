pub static SCHEMA_TS: &str = r#"import { object, string } from "yup";

import type { InferType } from "yup";

export const NAMESchema = object({
  field: string().required("required"),
});

export type NAMEValues = InferType<typeof NAMESchema>;

export const DEFAULTNAME_DASH_VALUES: NAMEValues = {
  field: "",
};
"#;

pub static SCHEMA: &str = r#"import { object, string } from "yup";

export const NAMESchema = object({
  field: string().required("required"),
});

export const DEFAULTNAME_DASH_VALUES = {
  field: "",
};
"#;

pub static PROPTYPES: &str = r#"export type {
  // NEXT_TYPE
} from "@schemas/NAMESPACE";
"#;

pub static NEW_IMPORT: &str = r#"export * from "./NAME_CAMEL";
"#;
pub static TYPE_EXPORT: &str = r#"NAMEValues,
  // NEXT_TYPE"#;