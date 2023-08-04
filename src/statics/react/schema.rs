pub static SCHEMA_TS: &str = r#"import { object, string } from "yup";

import type { InferType } from "yup";

export const NAME_CAPITALSchema = object({
  field: string().required("required"),
});

export type NAME_CAPITALValues = InferType<typeof NAME_CAPITALSchema>;

export const DEFAULT_NAME_DASH_VALUES: NAME_CAPITALValues = {
  field: "",
};
"#;

pub static SCHEMA: &str = r#"import { object, string } from "yup";

export const NAME_CAPITALSchema = object({
  field: string().required("required"),
});

export const DEFAULT_NAME_DASH_VALUES = {
  field: "",
};
"#;

pub static PROPTYPES: &str = r#"export type {
  /* NEXT_IMPORT */
} from "@schemas/NAMESPACE";
"#;
