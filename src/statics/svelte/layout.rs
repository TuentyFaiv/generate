pub static BARREL_STYLES_STYLED: &str = r#"export * as layout from "./NAMESPACE.layout.styles";
"#;

pub static BARREL_STYLES_CSS: &str = r#"import "./NAMESPACE.layoutEXT_STYLES";
"#;

pub static PROPTYPES: &str = r#"export {};
"#;

pub static SCRIPT_TS_STYLED: &str = r#"<script lang="ts">
	import { layout } from "@NAMESPACE/styles";
</script>
"#;

pub static SCRIPT_STYLED: &str = r#"<script>
	import { layout } from "@NAMESPACE/styles";
</script>
"#;

pub static SCRIPT_TS_CSS: &str = r#"<script lang="ts">
	import "@NAMESPACE/styles";
</script>
"#;

pub static SCRIPT_CSS: &str = r#"<script>
	import "@NAMESPACE/styles";
</script>
"#;

pub static LAYOUT_STYLED: &str = r#"SCRIPT
<div class={layout.NAME_LOWER}>
	<slot />
</div>
"#;

pub static LAYOUT_CSS: &str = r#"SCRIPT
<div class="NAME_LOWER">
	<slot />
</div>
"#;

pub static STYLES_EMOTION: &str = r#"import { css, cx } from "@emotion/css";

import * as responsive from "./NAMESPACE.layout.styles.responsive";

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
