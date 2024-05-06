pub static BARREL_STYLES_STYLED: &str = r#"export * as page from "./NAMESPACE.styles";
"#;

pub static BARREL_STYLES_CSS: &str = r#"import "./NAMESPACEEXT_STYLES";
"#;

pub static I18N_IMPORT: &str = r#"export * from "./i18n/store";
"#;

pub static PROPTYPES: &str = r#"export {};
"#;

pub static SCRIPT_TS_STYLED: &str = r#"<script lang="ts">
  import { i18n } from "@stores";

  import { page } from "@NAMESPACE/styles";

	import { SEO } from "@sharing/atoms";
</script>
"#;

pub static SCRIPT_STYLED: &str = r#"<script>
  import { i18n } from "@stores";

  import { page } from "@NAMESPACE/styles";

	import { SEO } from "@sharing/atoms";
</script>
"#;

pub static SCRIPT_TS_CSS: &str = r#"<script lang="ts">
  import { i18n } from "@stores";

  import "@NAMESPACE/styles";

	import { SEO } from "@sharing/atoms";
</script>
"#;

pub static SCRIPT_CSS: &str = r#"<script>
  import { i18n } from "@stores";

  import "@NAMESPACE/styles";

	import { SEO } from "@sharing/atoms";
</script>
"#;

pub static PAGE_CSS: &str = r#"SCRIPT
<SEO title={$i18n.t("NAME_LOWER:seo-title")} />

<section class="NAME_LOWER">
  <h1 class="NAME_LOWER__title">
    {$i18n.t("NAME_LOWER:seo-title")}
	</h1>
</section>
"#;

pub static PAGE_STYLED: &str = r#"SCRIPT
<SEO title={$i18n.t("NAME_LOWER:seo-title")} />

<section class={page.NAME_LOWER}>
  <h1 class={page.title}>
    {$i18n.t("NAME_LOWER:seo-title")}
	</h1>
</section>
"#;

pub static STYLES_EMOTION: &str = r#"import { css, cx } from "@emotion/css";

import * as responsive from "./NAMESPACE.styles.responsive";

export const NAME_LOWER = cx(
	css``,
	responsive.NAME_LOWER,
);

export const title = cx(
	css``,
	responsive.title,
);
"#;

pub static STYLES_EMOTION_RESPONSIVE: &str = r#"import { css } from "@emotion/css";
import { forsize } from "@mixins";

export const NAME_LOWER = forsize({
  "desktop-mid": css``,
  desktop: css``,
});

export const title = forsize({
  "desktop-mid": css``,
  desktop: css``,
});
"#;

pub static LOCALE: &str = r#"{
  "seo-title": "NAME_PASCAL"
}
"#;

pub static I18N: &str = r#"import { createI18nStore } from "svelte-i18next";
import i18next from "i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import Backend from "i18next-http-backend";
import { enUS, es } from "date-fns/locale";
import config from "@config";

i18next
  .use(Backend)
  .use(LanguageDetector)
  .init({
    debug: false,
    fallbackLng: config.i18n_fallback_lang,
    supportedLngs: config.i18n_langs,
    ns: [
      /* NEXT_LOCALE */
    ],
    defaultNS: "translation",
    load: "currentOnly",
    backend: {
      loadPath: "/locales/{{lng}}/{{ns}}.json",
    },
  });

const i18n = createI18nStore(i18next);

const locales = {
  "en-US": enUS,
  es,
};

export { i18next, i18n, locales };
"#;

pub static SVELTE_CONFIG: &str = r#"import adapter from "@sveltejs/adapter-static";
import preprocess from "svelte-preprocess";

/** @type {import("@sveltejs/kit").Config} */
const config = {
	// Consult https://github.com/sveltejs/svelte-preprocess
	// for more information about preprocessors
	preprocess: preprocess(),

	kit: {
		adapter: adapter(),
		alias: {
			// Assets
			"@images/*": "src/assets/images/*",
			"@icons/*": "src/assets/images/icons/*",
			// Logic
			"@config": "src/logic/config.ts",
			"@stores": "src/logic/stores/index.ts",
			"@actions": "src/logic/actions/index.ts",
			"@schemas/*": "src/logic/schemas/*",
			"@services/*": "src/logic/services/*",
			"@typing/*": "src/logic/typing/*",
			"@utils/*": "src/logic/utils/*",
			/* NEXT_ALIAS */
			// UI Sharing
			"@sharing/atoms": "src/ui/sharing/atoms/index.ts",
			"@sharing/molecules": "src/ui/sharing/molecules/index.ts",
			"@sharing/organisms": "src/ui/sharing/organisms/index.ts",
			"@styles": "src/ui/sharing/styles/globals.ts",
			"@mixins": "src/ui/sharing/styles/mixins.ts",
		}
	}
};

export default config;
"#;

pub static SVELTE_ALIAS: &str = r#"// UI NAME_PASCAL
			"@NAMESPACE/atoms": "src/ui/NAMESPACE/atoms/index.ts",
			"@NAMESPACE/molecules": "src/ui/NAMESPACE/molecules/index.ts",
			"@NAMESPACE/organisms": "src/ui/NAMESPACE/organisms/index.ts",
			"@NAMESPACE/styles": "src/ui/NAMESPACE/styles/index.ts",
      /* NEXT_ALIAS */"#;