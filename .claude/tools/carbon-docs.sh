#!/bin/sh
# Builds ~/carbon-docs for the carbon-expert agent: Carbon's website
# documentation as plain Markdown (MDX markup, demos and link targets removed,
# image alt text kept), an index of every page, and the @carbon/styles Sass
# source for exact values. Run it once per machine; it takes about a minute.
set -eu
site=5e9cd1da43c32d3d3b991dc947b427f674da61a8 # last commit before the site moved to a CMS
styles=1.117.0                                  # matches @carbon/themes in web/package.json
styles_sha512=1m/UH8aeSbDQ4PVIZ8no4QxvGhCswVV0tzMxdAaIrnRRtea6aMj6xQ3bKc1SR+F2SvpF2LJyut0Yd7hVQ4uoyQ==
out=$HOME/carbon-docs
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

git clone --quiet --filter=blob:none --no-checkout https://github.com/carbon-design-system/carbon-website.git "$work/site"
git -C "$work/site" sparse-checkout set --no-cone '/src/pages/**/*.mdx' '/src/data/guidelines/glossary.js'
git -C "$work/site" checkout --quiet "$site"

curl -fsSL -o "$work/styles.tgz" "https://registry.npmjs.org/@carbon/styles/-/styles-$styles.tgz"
[ "$(openssl dgst -sha512 -binary "$work/styles.tgz" | base64 -w0)" = "$styles_sha512" ] ||
	{ echo "styles checksum mismatch" >&2; exit 1; }

rm -rf "$out"
mkdir -p "$out/styles"
tar -xzf "$work/styles.tgz" -C "$out/styles" --strip-components=1 package/scss
cp "$work/site/src/data/guidelines/glossary.js" "$out/glossary.js"
python3 -I "$(dirname "$0")/carbon-docs.py" "$work/site/src/pages" "$out"
