pub static SCHEMA: &str = r#"import { string } from "yup";

export const NAME_CAPITALSchema = {
  field: string().required("required"),
};
"#;

pub static PROPTYPES: &str = r#"import { object } from "yup";
import { NAME_CAPITALSchema } from "@schemas/NAMESPACE";

import type { InferType } from "yup";

const NAME_LOWER = object(NAME_CAPITALSchema);
export type NAME_CAPITALValues = InferType<typeof NAME_LOWER>;
"#;
