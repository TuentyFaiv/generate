pub static SCHEMA_TS: &str = r#"import { object, string } from "yup";

import type { InferType } from "yup";

export const NAME_PASCALSchema = object({
  field: string().required("required"),
});

export type NAME_PASCALValues = InferType<typeof NAME_PASCALSchema>;

export const DEFAULT_NAME_CONSTANT_VALUES: NAME_PASCALValues = {
  field: "",
};
"#;

pub static SCHEMA: &str = r#"import { object, string } from "yup";

export const NAME_PASCALSchema = object({
  field: string().required("required"),
});

export const DEFAULT_NAME_CONSTANT_VALUES = {
  field: "",
};
"#;

pub static PROPTYPES: &str = r#"export type {
  /* NEXT_IMPORT */
} from "@schemas/NAMESPACE";
"#;
