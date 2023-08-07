pub static SERVICE_TS: &str = r#"import type {
  NAMEBody,
  NAMEParams,
  NAMEConfig,
  NAMEReturn,
} from "@typing/services/NAMESPACE";

import { api } from "@services/general";

export async function NAME_CAMEL(config: NAMEConfig) {
  const { payload } = await api.post<NAMEBody, NAMEReturn, NAMEParams>("/", config.body, {
    signal: config.signal,
    params: config.params,
  });

  return payload;
}
"#;
pub static SERVICE: &str = r#"import { api } from "@services/general";

export async function NAME_CAMEL(config) {
  const { payload } = await api.post("/", config.body, {
    signal: config.signal,
    params: config.params,
  });

  return payload;
}
"#;
pub static PROPTYPES: &str = r#"import type { ServiceGeneralConfig } from "@typing/globals/services";
// NEXT_TYPE"#;

pub static TYPE_EXPORT: &str = r#"
// NAME
// Change for interface if is an object
export type NAMEBody = null;

// Change for interface if is an object
export type NAMEParams = null;

// Change for interface if is an object
export type NAMEReturn = null;

export type NAMEConfig = ServiceGeneralConfig<NAMEBody, NAMEParams>;
// NEXT_TYPE"#;

pub static SERVICE_IMPORT: &str = r#"export * from "./NAME_CAMEL";
"#;

pub static INSTANCES: &str = r#"import config from "@config";

import Http from "./http";

export const api = Http.create(config.api);
export const registry = Http.create(config.registry_url, {
  params: {
    key: config.registry_key,
  },
});

export function setAuth(token: string) {
  api.setAuth(token);
}

export function setLang(lang: string) {
  api.setLang(lang);
}
"#;
