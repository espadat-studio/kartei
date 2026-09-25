import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightLlmsTxt from "starlight-llms-txt";

export default defineConfig({
  site: "https://kartei.espadat.com",
  integrations: [
    starlight({
      plugins: [starlightLlmsTxt()],
      title: "kartei",
      description: "Keyboard-driven editor for contacts stored as vCard files in a local vdir.",
      logo: { src: "./src/assets/mark.svg", alt: "Espadat" },
      head: [{ tag: "link", attrs: { rel: "icon", href: "/favicon.ico", sizes: "16x16 32x32" } }],
      customCss: ["@espadat/docs-theme/styles/theme.css"],
      components: {
        Footer: "@espadat/docs-theme/components/footer.astro",
        ThemeProvider: "@espadat/docs-theme/components/theme-provider.astro",
        ThemeSelect: "@espadat/docs-theme/components/theme-select.astro",
      },
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/espadat-studio/kartei" },
      ],
      sidebar: [
        { label: "Home", link: "/" },
        { label: "Getting Started", slug: "getting-started" },
        { label: "Keymap", slug: "keymap" },
        { label: "About", items: [{ slug: "about/license" }] },
      ],
    }),
  ],
});
