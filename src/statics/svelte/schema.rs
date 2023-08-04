pub static SCHEMA: &str = r#"import { string } from "yup";

export const NAME_CAPITALSchema = {
  field: string().required("required"),
};
"#;

pub static PROPTYPES: &str = r#"const NAME_CAMEL = object(NAME_CAPITALSchema);
export type NAME_CAPITALValues = InferType<typeof NAME_CAMEL>;

// NEXT_TYPE"#;

pub static PROPTYPES_IMPORTS: &str = r#"import { object } from "yup";
import {
  /* NEXT_IMPORT */
} from "@schemas/NAMESPACE";

import type { InferType } from "yup";

// PROPTYPES
"#;
