pub static COMP_PROPTYPES: &str = r#"export interface Props {
  prop: unknown;
}
"#;

pub static COMP_STYLES: &str = r#"import { css, cx } from "@emotion/css";

import * as responsive from "./NAME.styles.responsive";

export const NAME_LOWER = cx(css`

`, responsive.NAME_LOWER);
"#;

pub static COMP_STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/css";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static COMPONENT: &str = r#"SCRIPT

<div class={styles.NAME_LOWER}>
  {prop}
</div>
"#;

pub static SCRIPT_TS: &str = r#"<script lang="ts">
  import type { Props } from "./NAME.proptypes";

  import * as styles from "./NAME.styles";

  export let prop: Props["prop"];
</script>"#;

pub static SCRIPT: &str = r#"<script>
  import * as styles from "./NAME.styles";

  export let prop;
</script>"#;