use std::io::{self, Write};
use std::fs::File;
use anyhow::{Context, Result, Ok};
// use super::Template;

static PROPTYPES: &str = r#"export interface Props {
  prop: unknown;
}
"#;

static STYLES: &str = r#"import { css } from "@emotion/css";

import * as responsive from "./NAME.styles.responsive";

export const NAME_LOWER = css`
  ${responsive.NAME_LOWER}
`;

"#;

static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/css";
import { forsize } from "@mixins";

export const NAME_LOWER = css`
${forsize({ size: "desktop-mid", content: css`

` })}
`;

"#;

static COMPONENT_TS: &str = r#"<script lang="ts">
  import type { Props } from "./NAME.proptypes";

  import * as styles from "./NAME.styles";

  export let prop: Props["prop"];
</script>

<div class={styles.NAME_LOWER}>
  {prop}
</div>

"#;

static COMPONENT: &str = r#"<script>
  import * as styles from "./NAME.styles";

  export let prop;
</script>

<div class={styles.NAME_LOWER}>
  {prop}
</div>

"#;

pub fn generate(path: &str, name: &str, lang: &String) -> Result<()> {
  let is_ts = lang.as_str() == "typescript";

  let proptypes = PROPTYPES.to_string();
  let mut styles = STYLES.to_string();
  let mut responsive = STYLES_RESPONSIVE.to_string();
  
  let mut component = COMPONENT.to_string();
  let mut ext = ".js".to_string();

  if is_ts {
    component = COMPONENT_TS.to_string();
    ext = ".ts".to_string();
  }

  component = component.replace("NAME_LOWER", &name.to_lowercase());
  styles = styles.replace("NAME_LOWER", &name.to_lowercase());
  responsive = responsive.replace("NAME_LOWER", &name.to_lowercase());
  component = component.replace("NAME", name);
  styles = styles.replace("NAME", name);
  responsive = responsive.replace("NAME", name);

  let mut component_file = File::create(format!("{path}/{name}.svelte"))?;
  let mut styles_file = File::create(format!("{path}/{name}.styles{ext}"))?;
  let mut responsive_file = File::create(format!("{path}/{name}.styles.responsive{ext}"))?;
  
  component_file.write_all(component.as_bytes())?;
  styles_file.write_all(styles.as_bytes())?;
  responsive_file.write_all(responsive.as_bytes())?;

  if is_ts {
    let mut proptypes_file = File::create(format!("{path}/{name}.proptypes{ext}"))?;
    proptypes_file.write_all(proptypes.as_bytes())?;
  }

  Ok(())
}
