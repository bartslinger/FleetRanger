"""Converts Carbon's website MDX to plain Markdown and writes INDEX.md.

Usage: carbon-docs.py <carbon-website/src/pages> <output dir>
Called by carbon-docs.sh. Keeps all prose, tables, do/don't captions and image
alt text; drops imports, live demos, layout tags, link targets and the
feedback footer, which the agent would otherwise pay for on every read.
"""

import re
import sys
from pathlib import Path

SECTIONS = ('components', 'patterns', 'elements', 'guidelines', 'data-visualization')

LAYOUT_TAGS = (
    'Row|Column|Grid|PageDescription|InlineNotification|Caption|Title|div|span|'
    'GifPlayer|ImageGallery|ImageGalleryImage|Aside|AspectRatio|Accordion|'
    'AccordionItem|Tabs?|TabList|TabPanels?|ArticleCard|ResourceCard|MdxIcon|'
    'ExpressiveListContainer|UnorderedList|OrderedList|StructuredList\\w*|'
    'ArtDirection|Preview|DoDontRow|ImageCard|OverviewCard|Link'
)


def do_dont(match):
    attrs = match.group(1)
    kind = "DON'T" if 'dont' in attrs else 'DO' if 'type="do"' in attrs else 'DO/DON\'T'
    caption = re.search(r'caption="([^"]*)"', attrs)
    return f'[{kind}: {caption.group(1)}]' if caption else f'[{kind}]'


def convert(text):
    front = re.match(r'---\n(.*?)\n---\n', text, re.S)
    meta = front.group(1) if front else ''
    title = re.search(r'^title:\s*(.*)$', meta, re.M)
    description = re.search(r'^description:\s*(.*?)(?=^\S|\Z)', meta, re.M | re.S)
    title = title.group(1).strip() if title else ''
    description = ' '.join(description.group(1).split()) if description else ''
    if front:
        text = text[front.end():]

    text = re.sub(r'^import .*?;\s*$', '', text, flags=re.M)
    text = re.sub(r'\{/\*.*?\*/\}|<!--.*?-->', '', text, flags=re.S)
    for tag in ('StorybookDemo', 'A11yStatus', 'KalturaVideo', 'Video', 'ChartDemoGroup'):
        text = re.sub(rf'<{tag}\b.*?(/>|</{tag}>)', '', text, flags=re.S)
    text = re.sub(r'<AnchorLinks.*?</AnchorLinks>', '', text, flags=re.S)
    text = re.sub(r'!\[([^\]]*)\]\([^)]*\)',
                  lambda m: f'[image: {m.group(1)}]' if m.group(1).strip() else '', text)
    text = re.sub(r'<img[^>]*?alt="([^"]*)"[^>]*>', r'[image: \1]', text)
    text = re.sub(r'\[([^\]]+)\]\((?:[^()]|\([^)]*\))*\)', r'\1', text)
    text = re.sub(r'<DoDont\b([^>]*?)/?>', do_dont, text, flags=re.S)
    text = re.sub(r'</DoDont>', '', text)
    text = re.sub(r'<ListItem>\s*', '\n- ', text)
    text = re.sub(rf'</?({LAYOUT_TAGS}|ListItem)\b[^>]*?/?>', '', text)
    text = re.sub(r'^## Feedback\b.*', '', text, flags=re.S | re.M)
    lines = []
    for line in text.splitlines():
        if line.lstrip().startswith('|'):
            line = re.sub(r' {2,}', ' ', re.sub(r'-{3,}', '---', line))
        lines.append(line.rstrip())
    text = re.sub(r'\n{3,}', '\n\n', '\n'.join(lines)).strip()
    head = f'# {title}\n\n{description}\n\n' if title else ''
    return title, description, head + text + '\n'


def main():
    pages, out = Path(sys.argv[1]), Path(sys.argv[2])
    # One index entry per topic directory: its pages with sizes, then the
    # sections of its main page, so the agent can pick files without searching.
    topics = {}
    for section in SECTIONS:
        for src in sorted((pages / section).rglob('*.mdx')):
            if src.stem == 'code':
                continue
            rel = src.relative_to(pages).with_suffix('.md')
            title, description, body = convert(src.read_text())
            dst = out / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_text(body)
            topic = topics.setdefault(rel.parent, {'title': title, 'description': description,
                                                    'pages': [], 'sections': []})
            topic['pages'].append(f'{rel.stem} {len(body) // 4000 + 1}k')
            if src.stem in ('usage', 'index', 'overview') or rel.parent == Path(section):
                topic['sections'] += re.findall(r'^## (.+)$', body, re.M)[:12]
    lines = []
    for path, topic in topics.items():
        sections = '; '.join(dict.fromkeys(topic['sections']))
        lines.append(f'- {path}/ ({", ".join(topic["pages"])}) {topic["title"]}: '
                     f'{topic["description"]} [{sections}]')
    (out / 'INDEX.md').write_text(
        '# Carbon documentation index\n\nOne line per topic: directory, (pages and their '
        'size in tokens), title: description [sections].\n\n' + '\n'.join(lines) + '\n')


main()
