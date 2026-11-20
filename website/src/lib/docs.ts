/**
 * The documentation: Markdown files in website/docs/<section>/, turned into
 * HTML when the site is built. A file's name gives its place and address
 * (`02-sessions.md` is the second page, at /docs/<section>/sessions), its
 * first line `# …` its title.
 *
 * Links between pages are written as GitHub reads them (`02-sessions.md`,
 * `../self-hosting/01-requirements.md#dji`), so the files also work in the
 * repository; here they become the pages' addresses.
 */
import { Marked, type Tokens } from 'marked';

export interface Section {
	slug: string;
	title: string;
	pages: Page[];
}

export interface Page {
	section: string;
	slug: string;
	title: string;
	/** The page's HTML, its title left out (the layout shows it). */
	html: string;
}

/** The sections, in the order the navigation shows them. */
const SECTIONS: { slug: string; title: string }[] = [
	{ slug: 'user', title: 'Using FleetRanger' },
	{ slug: 'self-hosting', title: 'Self-hosting' }
];

const files = import.meta.glob<string>('/docs/*/*.md', {
	query: '?raw',
	import: 'default',
	eager: true
});

const FILE = /^\/docs\/([a-z-]+)\/(\d+)-([a-z0-9-]+)\.md$/;
/** Pictures beside the pages, in docs/<section>/images/, written in a page
 * as `![…](images/name.webp)`; the site serves them under hashed names. */
const images = import.meta.glob<string>('/docs/*/images/*.{png,webp}', {
	query: '?url',
	import: 'default',
	eager: true
});

const LINK = /^(?:\.\.\/([a-z-]+)\/)?\d+-([a-z0-9-]+)\.md(#[a-z0-9-]+)?$/;

/** A heading's anchor: its text in lower case, words joined by dashes. */
function anchor(text: string): string {
	return text
		.toLowerCase()
		.replace(/<[^>]*>/g, '')
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-|-$/g, '');
}

function escape(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

function render(section: string, markdown: string): string {
	const marked = new Marked({
		renderer: {
			heading({ tokens, depth }: Tokens.Heading) {
				const text = this.parser.parseInline(tokens);
				return `<h${depth} id="${anchor(text)}">${text}</h${depth}>\n`;
			},
			// Focusable, so a box that scrolls sideways scrolls with the keyboard.
			code({ text }: Tokens.Code) {
				return `<pre tabindex="0"><code>${escape(text)}</code></pre>\n`;
			},
			image({ href, text }: Tokens.Image) {
				const url = images[`/docs/${section}/${href}`];
				if (!url) throw new Error(`docs/${section}: no picture ${href}`);
				return `<img src="${url}" alt="${escape(text)}" loading="lazy" />`;
			},
			link({ href, tokens }: Tokens.Link) {
				const text = this.parser.parseInline(tokens);
				const page = LINK.exec(href);
				if (page) {
					href = `/docs/${page[1] ?? section}/${page[2]}${page[3] ?? ''}`;
				}
				return `<a href="${href}">${text}</a>`;
			}
		}
	});
	return marked.parse(markdown, { async: false });
}

function load(): Section[] {
	const sections = SECTIONS.map((s) => ({ ...s, pages: [] as (Page & { order: number })[] }));
	for (const [path, markdown] of Object.entries(files)) {
		const name = FILE.exec(path);
		const section = sections.find((s) => s.slug === name?.[1]);
		if (!name || !section) {
			throw new Error(`${path}: not a docs page (website/docs/<section>/<NN>-<name>.md)`);
		}
		const [first, ...rest] = markdown.split('\n');
		if (!first?.startsWith('# ')) {
			throw new Error(`${path}: the first line must be the title, "# …"`);
		}
		section.pages.push({
			section: section.slug,
			slug: name[3] ?? '',
			title: first.slice(2).trim(),
			html: render(section.slug, rest.join('\n')),
			order: Number(name[2])
		});
	}
	for (const section of sections) {
		section.pages.sort((a, b) => a.order - b.order);
	}
	return sections;
}

export const sections: Section[] = load();

export function find(section: string, slug: string): Page | undefined {
	return sections.find((s) => s.slug === section)?.pages.find((p) => p.slug === slug);
}
