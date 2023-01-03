use chrono::Utc;
use clap::Parser;
use select::{document::Document, predicate::Name};
use std::fs::File;
use std::io::Write;
use std::{collections::HashSet, future::Future};

#[derive(Debug, Parser)]
struct Arguments {
    #[clap(short = 't', long = "site", env = "SITE_URI")]
    uri: String,

    #[clap(short = 'x', long = "exclude")]
    exclude: Option<Vec<String>>,
}

struct SitemapGen {
    base_uri: String,
    exclude_paths: HashSet<String>,
    visited_paths: HashSet<String>,
    client: surf::Client,
}

impl SitemapGen {
    fn new(base_uri: String, exclude_paths: Vec<String>) -> Self {
        SitemapGen {
            base_uri,
            exclude_paths: HashSet::from_iter(exclude_paths),
            visited_paths: HashSet::<String>::new(),
            client: surf::Client::new().with(surf::middleware::Redirect::new(5)),
        }
    }

    fn internal_link(&self, link: &str) -> Option<String> {
        if link.starts_with('/') {
            Some(link.to_string())
        } else if link.starts_with(&self.base_uri) {
            Some(link.replace(&self.base_uri, ""))
        } else {
            None
        }
    }

    fn is_exclude_link(&self, link: &str) -> bool {
        self.exclude_paths.iter().any(|x| link.starts_with(x))
    }

    fn get_page(&self, path: String) -> impl Future<Output = Result<String, surf::Error>> {
        let uri = if path.starts_with("http://") || path.starts_with("https://") {
            path
        } else {
            format!("{}{}", self.base_uri, path)
        };
        self.client.get(uri).recv_string()
    }

    fn extract_links(&self, body: &str) -> HashSet<String> {
        let doc = Document::from(body);
        doc.find(Name("a"))
            .filter_map(|x| x.attr("href"))
            .filter_map(|x| self.internal_link(x))
            .filter(|x| !self.is_exclude_link(x))
            .collect()
    }

    async fn visit_paths(&self, paths: HashSet<String>) -> Vec<Result<String, surf::Error>> {
        let mut handles = vec![];
        for path in paths {
            let request = self.get_page(path);
            handles.push(async_std::task::spawn(request));
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await);
        }
        results
    }

    fn collect_all_paths(&mut self, path_to_visit: HashSet<String>) -> HashSet<String> {
        if path_to_visit.is_empty() {
            return path_to_visit;
        }

        let results = async_std::task::block_on(self.visit_paths(path_to_visit.clone()));

        let mut paths_to_visit_next: HashSet<String> = HashSet::new();
        for r in results {
            match r {
                Ok(body) => {
                    for p in self.extract_links(body.as_str()) {
                        paths_to_visit_next.insert(p);
                    }
                }
                Err(e) => eprintln!(">>> {:?}", e),
            }
        }

        for p in path_to_visit.iter() {
            self.visited_paths.insert(p.to_string());
        }

        paths_to_visit_next
    }

    fn collect_paths(&mut self) -> &Self {
        let mut paths_to_visit = HashSet::from(["/".to_string()]);
        loop {
            paths_to_visit = self.collect_all_paths(paths_to_visit);

            // remove already visited
            for p in &self.visited_paths {
                paths_to_visit.remove(p);
            }

            if paths_to_visit.is_empty() {
                break;
            }
        }
        self
    }

    fn write_xml(&self) -> std::io::Result<()> {
        let mut f = File::create("sitemap.xml")?;
        f.write_all(b"<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n")?;
        for p in &self.visited_paths {
            f.write_all(b"  <url>\n")?;
            f.write_fmt(format_args!("    <loc>{}{}</loc>\n", self.base_uri, p))?;
            f.write_fmt(format_args!(
                "    <lastmod>{}</lastmod>\n",
                Utc::now().to_rfc3339()
            ))?;
            f.write_all(b"    <changefreq>monthly</changefreq>\n")?;
            f.write_all(b"    <priority>0.5</priority>\n")?;
            f.write_all(b"  </url>\n")?;
        }
        f.write_all(b"</urlset>")?;
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let args = Arguments::parse();
    let uri = args.uri;
    println!("Target site: {}", uri);
    let exclude_paths;
    if let Some(exclude) = args.exclude {
        exclude_paths = exclude;
        println!("excludes: {:?}", exclude_paths);
    } else {
        exclude_paths = vec![];
    }

    SitemapGen::new(uri, exclude_paths)
        .collect_paths()
        .write_xml()?;

    Ok(())
}
