pub static PROPTYPES: &str = r#"export {};
"#;

pub static LAYOUT_TS: &str = r#"<script lang="ts">
	import { layout } from "@NAME_LOWER/styles";
</script>

<div class={layout.main}>
	<slot />
</div>
"#;

pub static LAYOUT: &str = r#"<script>
	import { layout } from "@NAME_LOWER/styles";
</script>

<div class={layout.main}>
	<slot />
</div>
"#;

pub static STYLES_IMPORT: &str = r#"export * as layout from "./NAME_LOWER.layout.styles";
"#;

pub static STYLES: &str = r#"import { css } from "@emotion/css";

import * as responsive from "./NAME_LOWER.layout.styles.responsive";

export const NAME_LOWER = css`
${responsive.NAME_LOWER}
`;
"#;

pub static STYLES_RESPONSIVE: &str = r#"import { css } from "@emotion/css";
import { forsize } from "@mixins";

export const NAME_LOWER = css`
${forsize({ size: "desktop-mid", content: css`

` })}
`;
"#;
