import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import vercel from "@astrojs/vercel/static";

import tailwind from "@astrojs/tailwind";

// https://astro.build/config
export default defineConfig({
  site: "https://cli.tuentyfaiv.com",
  integrations: [starlight({
    title: 'TFverse',
    social: {
      github: 'https://github.com/TuentyFaiv/generate'
    },
    sidebar: [{
      label: 'Guides',
      items: [
      // Each item here is one entry in the navigation menu.
      {
        label: 'Example Guide',
        link: '/guides/example/'
      }]
    }, {
      label: 'Reference',
      autogenerate: {
        directory: 'reference'
      }
    }],
    defaultLocale: "root",
    locales: {
      root: {
        label: "English",
        lang: "en"
      },
      es: {
        label: "Español",
        lang: "es-MX"
      }
    }
  }), tailwind()],
  output: "static",
  adapter: vercel({
    webAnalytics: {
      enabled: true
    },
    imagesConfig: {
      sizes: [320, 480, 768, 1024, 1200, 1600, 1920],
      domains: ["cli.tuentyfaiv.com"]
    },
    imageService: true
  })
});