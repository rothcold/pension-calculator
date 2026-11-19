# Alloy CSS subset

Source: `https://github.com/alloyhtml/css`, commit `5232d781cac670d58f21b7f327c0eaf341e2ec71`, file `dist/alloy.css` (211,484 bytes; SHA-256 `12cab0c9d44b50f2a432acf819d3809e8dcc3b9077e3817932ea534ae8de7022`).

`alloy.css` contains the upstream MIT license, then these complete rules copied verbatim, in source order. Line ranges are inclusive:

- 126–152: border-box, element and fieldset resets, body tokens.
- 164–177: native control fonts, table borders, keyboard focus.
- 184–191: `.container-1320` page edges.
- 229–258: `.button`, including pointer and disabled states.
- 294–337: `.field`, including hover, focus, invalid and disabled states.

The local file is 4,202 bytes; SHA-256 `7928005dcb970618316095d18020a9c378fea4ca988e6afc6e57ea5379e605e0`. Keep the license in the CSS so both distributions carry it. To update, review the upstream compiled rules, replace the excerpts, and update this manifest and the digest check in `tests/site.cjs`.

`teal.css` loads second. It defines the color and font tokens, narrows the container, and increases control height and keyboard focus contrast. The app keeps native checkboxes, validation and disclosure markers. It does not need Alloy's alternate themes, utility catalog, dialogs or menus. Neither file requires Sass, JavaScript, npm, a CDN or remote fonts.
