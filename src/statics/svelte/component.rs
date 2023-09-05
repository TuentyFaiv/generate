pub static PROPTYPES: &str = r#"export interface Props {
  prop: unknown;
}
"#;

pub static STYLES_EMOTION: &str = r#"import { css, cx } from "@emotion/css";

import * as responsive from "./NAME_PASCAL.styles.responsive";

export const NAME_LOWER = cx(
  css``,
  responsive.NAME_LOWER,
);
"#;

pub static STYLES_EMOTION_RESPONSIVE: &str = r#"import { css } from "@emotion/css";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static COMPONENT_CSS: &str = r#"SCRIPT

<div class="NAME_LOWER">
  {prop}
</div>
"#;

pub static COMPONENT_STYLED: &str = r#"SCRIPT

<div class={styles.NAME_LOWER}>
  {prop}
</div>
"#;

pub static SCRIPT_TS_CSS: &str = r#"<script lang="ts">
  import type { Props } from "./NAME_PASCAL.proptypes";

  import "./NAME_PASCALEXT_STYLES";

  export let prop: Props["prop"];
</script>"#;

pub static SCRIPT_CSS: &str = r#"<script>
  import "./NAME_PASCALEXT_STYLES";

  export let prop;
</script>"#;

pub static SCRIPT_TS_STYLED: &str = r#"<script lang="ts">
  import type { Props } from "./NAME_PASCAL.proptypes";

  import * as styles from "./NAME_PASCAL.styles";

  export let prop: Props["prop"];
</script>"#;

pub static SCRIPT_STYLED: &str = r#"<script>
  import * as styles from "./NAME_PASCAL.styles";

  export let prop;
</script>"#;
