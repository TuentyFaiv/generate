pub static SERVICE_TS: &str = r#"import { throwError } from "@utils/errors";

import type { NAMEConfig, NAMEReturn } from "@typing/services/NAME_CAMEL";

import { api } from "./instances";

export async function NAME_CAMEL(config?: NAMEConfig): Promise<NAMEReturn> {
  const { payload } = await api.get<NAMEReturn>("/", {
    signal: config.signal
  });

  return payload;
}
"#;
pub static SERVICE: &str = r#"import { throwError } from "@utils/errors";

import { api } from "./instances";

export async function NAME_CAMEL(config) {
  const { payload } = await api.get("/", {
    signal: config.signal
  });

  return payload;
}
"#;
pub static PROPTYPES: &str = r#"export interface NAMEConfig {
  signal?: AbortSignal;
}

// Change for interface if is an object
export type NAMEReturn = null;
"#;
pub static INSTANCES: &str = r#"import { Http } from "@tuentyfaiv/http";
import config from "@config";

export const api = Http.create(config.api);
"#;
