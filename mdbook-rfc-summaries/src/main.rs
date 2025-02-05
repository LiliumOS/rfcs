use std::io;
use std::path::{Path, PathBuf};

use mdbook::BookItem;
use mdbook::book::Chapter;
use mdbook::preprocess::CmdPreprocessor;
use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag, TagEnd};

fn push_chapter<'a>(
    v: &'a mut Vec<BookItem>,
    base: &Path,
    p: PathBuf,
    link: &str,
    parents: &[String],
) -> io::Result<impl Iterator<Item = Event<'static>>> {
    let mut content = std::fs::read_to_string(base)?;
    content.push_str("\n!{#copyright}");

    let lines = content.lines();

    let mut title = None::<&str>;

    for line in lines {
        if let Some(l) = line.strip_prefix('#') {
            if l.starts_with('#') {
                continue;
            }

            let l = l.split_once('{').map_or(l, |(l, _)| l);

            title = Some(l.trim());
            break;
        }
    }

    let title = title.map_or_else(|| String::new(), str::to_string);

    let ch = Chapter {
        name: title,
        content,
        number: None,
        sub_items: Vec::new(),
        path: Some(p.clone()),
        source_path: Some(p.clone()),
        parent_names: parents.to_vec(),
    };

    v.push(BookItem::Chapter(ch));

    match v.last() {
        Some(BookItem::Chapter(Chapter {
            path: Some(path),
            name,
            ..
        })) => Ok([
            Event::Start(Tag::Item),
            Event::Start(Tag::Link {
                link_type: pulldown_cmark::LinkType::Inline,
                dest_url: CowStr::Borrowed(link).into_static(),
                title: CowStr::Borrowed(name).into_static(),
                id: CowStr::Borrowed(""),
            }),
            Event::Text(CowStr::Borrowed(name).into_static()),
            Event::End(TagEnd::Link),
            Event::End(TagEnd::Item),
        ]
        .into_iter()),
        _ => unreachable!(),
    }
}

fn main() -> io::Result<()> {
    let mut args = std::env::args();
    args.next();

    match args.next().as_deref() {
        Some("supports") => return Ok(()),
        Some(s) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unexpected argument {s}"),
            ));
        }
        None => {}
    }

    let (ctx, mut book) = CmdPreprocessor::parse_input(std::io::stdin())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let parent = [String::from("rfcs")];
    // let mut rfcs = Vec::new();

    let mut root = PathBuf::new();
    root.push(&ctx.config.book.src);
    root.push("rfcs");
    let mut subchapters = Vec::new();
    let mut index_file = String::new();
    let mut state = pulldown_cmark_to_cmark::cmark(
        [
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                id: None,
                classes: Vec::new(),
                attrs: Vec::new(),
            }),
            Event::Text(CowStr::Borrowed("RFC Index")),
            Event::End(TagEnd::Heading(HeadingLevel::H1)),
            Event::Start(Tag::List(Some(1))),
        ]
        .into_iter(),
        &mut index_file,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    for item in std::fs::read_dir(&root)? {
        let item = item?;
        let base_path = item.path();
        let link = base_path
            .strip_prefix(&ctx.config.book.src)
            .and_then(|p| p.strip_prefix("rfcs"))
            .expect("Uh....")
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        let mdbook_path = base_path
            .strip_prefix(&ctx.config.book.src)
            .expect("Uh...")
            .to_path_buf();

        state = pulldown_cmark_to_cmark::cmark_resume(
            push_chapter(&mut subchapters, &base_path, mdbook_path, link, &parent)?,
            &mut index_file,
            Some(state),
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    pulldown_cmark_to_cmark::cmark_resume(
        [
            Event::End(TagEnd::List(true)),
            Event::Text(CowStr::Borrowed("!{#copyright}")),
        ]
        .into_iter(),
        &mut index_file,
        Some(state),
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let path = PathBuf::from("rfcs/index.md");

    let index_ch = Chapter {
        name: String::from("RFCs Index"),
        content: index_file,
        number: None,
        sub_items: subchapters,
        path: Some(path),
        source_path: None,
        parent_names: Vec::new(),
    };
    'a: {
        for (i, r) in book.sections.iter_mut().enumerate() {
            match r {
                BookItem::PartTitle(title) if title.contains("#rfc-index") => {
                    if let Some(l) = title.find('{') {
                        title.truncate(l);
                    }
                    book.sections.insert(i + 1, BookItem::Chapter(index_ch));
                    break 'a;
                }
                _ => {}
            }
        }

        book.sections.push(BookItem::Chapter(index_ch));
    }

    serde_json::to_writer(std::io::stdout(), &book)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}
