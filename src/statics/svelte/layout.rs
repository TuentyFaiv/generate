pub static PROPTYPES: &str = r#"export {};
"#;

pub static SCRIPT_TS: &str = r#"<script lang="ts">
	import { layout } from "@NAME_LOWER/styles";
</script>
"#;

pub static SCRIPT: &str = r#"<script>
	import { layout } from "@NAME_LOWER/styles";
</script>
"#;

pub static LAYOUT: &str = r#"SCRIPT
<div class={layout.NAME_LOWER}>
	<slot />
</div>
"#;

pub static STYLES: &str = r#"import { css, cx } from "@emotion/css";

import * as responsive from "./NAME_LOWER.layout.styles.responsive";

export const NAME_LOWER = cx(
	css``,
	responsive.NAME_LOWER,
);
"#;

pub static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/css";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;
