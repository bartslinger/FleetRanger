// Turns a screenshot into the WebP files the page uses, one per width:
//   node scripts/image.js <source.png> <name> <width>...
// writes src/lib/images/<name>-<width>.webp. The sources aren't kept in the
// repository; only these files are.

import sharp from 'sharp';

const [source, name, ...widths] = process.argv.slice(2);
if (!source || !name || widths.length === 0) {
	console.error('usage: node scripts/image.js <source.png> <name> <width>...');
	process.exit(1);
}

for (const width of widths.map(Number)) {
	const out = `src/lib/images/${name}-${width}.webp`;
	const info = await sharp(source).resize({ width }).webp({ quality: 82 }).toFile(out);
	console.log(`${out}: ${info.width}x${info.height}, ${Math.round(info.size / 1024)} kB`);
}
