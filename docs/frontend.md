# Frontend localization and imagery

The Vue 3 UI supports Simplified Chinese, English, French and Japanese. The header language selector updates existing pages immediately and remembers the selection in local storage when available. First-time visitors use a supported browser language, falling back to Chinese. The document language, title, description, dates and number formats follow the selected locale.

`frontend/src/i18n.ts` contains the landing-page and account messages. `frontend/src/locales/workflow.ts` contains the food-sharing, handoff and administration messages; its tuples are ordered **Chinese, English, French, Japanese**. Add all four translations and keep named interpolation placeholders identical. Render messages using `t(key, params)`, without HTML interpolation. API error codes are localized in `errorMessage`; existing station notifications are mapped by their known system message. User-authored food descriptions, organization names and dispute statements are preserved verbatim.

Run `npm run test:i18n` and `npm run build` in `frontend/`. The localization check catches missing/duplicate keys, mismatched parameters and Chinese interface strings accidentally left in Vue pages. Browser validation should include long French labels, mobile navigation, form labels, handoff notices and persisted language selection.

The homepage uses an illustrative **food photograph**, not a photograph of a RescueProof delivery. Source: [Unsplash image](https://images.unsplash.com/photo-1490645935967-10de6ba17061), downloaded on 2026-10-21. See the [Unsplash license](https://unsplash.com/license). Locally bundled WebP renditions are stored in `frontend/public/images/food-table-640.webp` and `food-table-1280.webp`. Native `srcset` lets the browser select a rendition for its viewport and pixel density; explicit sizing reserves the image area and the hero is loaded with high priority. No runtime image CDN or image-generation API is required.

The former CSS food illustration has been removed. Food listing cards use abstract category icons and do not pretend to show a photograph of each donated batch.
